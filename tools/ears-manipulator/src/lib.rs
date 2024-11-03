use std::{borrow::Borrow, io::Cursor};

use ears_rs::{
    alfalfa::{self, AlfalfaData},
    features::EarsFeatures,
    parser::{v0::writer::EarsWriterV0, v1::writer::EarsWriterV1, EarsFeaturesWriter, EarsParser}, utils::{self, EarsEmissivePalette},
};
use image::{ImageFormat, RgbaImage};
use js_sys::Uint8Array;
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
pub fn apply_features(skin_data: &[u8], features: JsValue) -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();

    let wasm_features: WasmEarsFeatures = serde_wasm_bindgen::from_value(features)?;
    
    let mut skin_image = image::load_from_memory(skin_data)?.into_rgba8();
    
    #[cfg(feature = "template")]
    {
        template::apply_template(&mut skin_image, wasm_features.borrow())?;
    }

    let features: EarsFeatures = wasm_features.clone().into();
    let emissive_palette: EarsEmissivePalette = wasm_features.borrow().into();
    let alfalfa_data: AlfalfaData = wasm_features.into();
    
    let writer = match features.data_version {
        0 => |image: &mut RgbaImage, features: &EarsFeatures| EarsWriterV0::write(image, features),
        _ => |image: &mut RgbaImage, features: &EarsFeatures| EarsWriterV1::write(image, features),
    };
    
    (writer)(&mut skin_image, &features)?;

    if !alfalfa_data.is_empty() {
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
