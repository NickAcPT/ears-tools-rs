use std::io::Cursor;

use ears_rs::{
    features::EarsFeatures,
    parser::{v1::writer::EarsWriterV1, EarsFeaturesWriter, EarsParser},
};
use image::ImageFormat;
use js_sys::Uint8Array;
use js_utils::JsResult;
use wasm_bindgen::prelude::*;

use crate::model::WasmEarsFeatures;

mod model;

#[wasm_bindgen]
pub fn get_ears_features(skin_data: &[u8]) -> JsResult<JsValue> {
    console_error_panic_hook::set_once();

    let skin_image = image::load_from_memory(skin_data)?.into_rgba8();

    let features = EarsParser::parse(&skin_image)?;

    let value = features
        .map(|f| Into::<WasmEarsFeatures>::into(f))
        .map(|f| serde_wasm_bindgen::to_value(&f))
        .transpose()?
        .unwrap_or(JsValue::NULL);

    return Ok(value);
}

#[wasm_bindgen]
pub fn apply_features(skin_data: &[u8], features: JsValue) -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();

    let features: WasmEarsFeatures = serde_wasm_bindgen::from_value(features)?;
    let features: EarsFeatures = features.into();

    let mut skin_image = image::load_from_memory(skin_data)?.into_rgba8();

    EarsWriterV1::write(&mut skin_image, &features)?;

    let mut bytes = Vec::new();
    {
        skin_image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    }

    Ok(Uint8Array::from(bytes.as_slice()))
}
