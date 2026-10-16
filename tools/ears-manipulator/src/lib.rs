use std::{borrow::Borrow, io::Cursor};

use ears_rs::{
    alfalfa::{self, AlfalfaData},
    features::{DataVersion, EarsFeatures},
    parser::{v0::writer::EarsWriterV0, v1::writer::EarsWriterV1, EarsFeaturesWriter, EarsParser},
    utils::{self, EarsEmissivePalette},
};
use image::{ImageFormat, RgbaImage};
use js_sys::{Array, Uint8Array};
use js_utils::JsResult;
use wasm_bindgen::prelude::*;

use crate::model::WasmEarsFeatures;

mod model;

#[cfg(feature = "template")]
mod template;

#[wasm_bindgen]
pub fn get_ears_features(skin_data: &[u8]) -> JsResult<JsValue> {
    console_error_panic_hook::set_once();

    let skin_image = image::load_from_memory(skin_data)?.into_rgba8();

    let features = EarsParser::parse(&skin_image)?;
    let alfalfa = alfalfa::read_alfalfa(&skin_image)?;

    let emissive_palette = utils::extract_emissive_palette(&skin_image)?;

    let value = features
        .map(|f| Into::<WasmEarsFeatures>::into(f))
        .map(|f| f.with_alfalfa(alfalfa))
        .map(|f| f.with_emissive(emissive_palette))
        .map(|f| serde_wasm_bindgen::to_value(&f))
        .transpose()?
        .unwrap_or(JsValue::NULL);

    return Ok(value);
}

#[wasm_bindgen]
pub fn get_template_skin(features: JsValue, slim_arms: bool) -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();

    let wasm_features: WasmEarsFeatures = serde_wasm_bindgen::from_value(features)?;

    let mut skin_image = RgbaImage::new(64, 64);

    #[cfg(feature = "template")]
    {
        template::apply_template(&mut skin_image, wasm_features.borrow(), true, slim_arms)?;
    }

    encode_png(&skin_image)
}

/// Sample wings texture laid out for `mode` (a `WingsMode` value).
#[cfg(feature = "template")]
#[wasm_bindgen]
pub fn get_template_wings(mode: JsValue) -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();

    let mode: model::WasmWingsMode = serde_wasm_bindgen::from_value(mode)?;

    encode_png(&template::wings_template(mode.into()))
}

/// Sample cape texture in the Ears cape layout.
#[cfg(feature = "template")]
#[wasm_bindgen]
pub fn get_template_cape() -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();

    encode_png(&template::cape_template())
}

fn encode_png(image: &RgbaImage) -> JsResult<Uint8Array> {
    let mut bytes = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;

    Ok(Uint8Array::from(bytes.as_slice()))
}

#[wasm_bindgen]
pub fn get_locator_sprites(atlas_data: &[u8]) -> JsResult<Array> {
    console_error_panic_hook::set_once();

    let atlas = image::load_from_memory(atlas_data)?.into_rgba8();

    let sprites = utils::extract_locator_sprites(&atlas).ok_or_else(|| {
        JsError::new(&format!(
            "Locator bar icons must be {}x{} pixels (got {}x{})",
            utils::LOCATOR_ATLAS_WIDTH,
            utils::LOCATOR_ATLAS_HEIGHT,
            atlas.width(),
            atlas.height()
        ))
    })?;

    Ok(sprites.iter().map(|s| Uint8Array::from(s.as_raw().as_slice())).collect())
}

#[wasm_bindgen]
pub fn tint_locator_atlas(atlas_data: &[u8], color: u32) -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();

    let mut atlas = image::load_from_memory(atlas_data)?.into_rgba8();
    let [_, r, g, b] = color.to_be_bytes();

    for pixel in atlas.pixels_mut() {
        for (channel, tint) in pixel.0.iter_mut().zip([r, g, b]) {
            *channel = ((*channel as u32 * tint as u32 + 127) / 255) as u8;
        }
    }

    let mut bytes = Vec::new();
    atlas.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;

    Ok(Uint8Array::from(bytes.as_slice()))
}

#[wasm_bindgen]
pub fn apply_features(
    skin_data: &[u8],
    features: JsValue,
    slim_arms: bool,
) -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();

    let wasm_features: WasmEarsFeatures = serde_wasm_bindgen::from_value(features)?;

    let mut skin_image = image::load_from_memory(skin_data)?.into_rgba8();
    let had_alfalfa = matches!(alfalfa::read_alfalfa(&skin_image), Ok(Some(_)));

    #[cfg(feature = "template")]
    {
        if wasm_features.apply_template {
            template::apply_template(&mut skin_image, wasm_features.borrow(), false, slim_arms)?;
        }
    }

    let features: EarsFeatures = wasm_features.clone().into();
    let emissive_palette: EarsEmissivePalette = wasm_features.borrow().into();
    let alfalfa_data: AlfalfaData = wasm_features.into();

    let writer = match features.data_version {
        DataVersion::V0 => {
            |image: &mut RgbaImage, features: &EarsFeatures| EarsWriterV0::write(image, features)
        }
        DataVersion::V1(_) | DataVersion::Custom(_) => {
            |image: &mut RgbaImage, features: &EarsFeatures| EarsWriterV1::write(image, features)
        }
    };

    (writer)(&mut skin_image, &features)?;

    if had_alfalfa || !alfalfa_data.is_empty() {
        alfalfa::write_alfalfa(&alfalfa_data, &mut skin_image)?;
    }

    if !emissive_palette.0.is_empty() {
        utils::write_emissive_palette(&mut skin_image, &emissive_palette)?;
    }

    let mut bytes = Vec::new();
    {
        skin_image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    }

    Ok(Uint8Array::from(bytes.as_slice()))
}
