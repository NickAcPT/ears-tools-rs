use js_sys::Uint8Array;
use js_utils::JsResult;
use wasm_bindgen::prelude::*;

use crate::model::WasmEarsFeatures;

mod model;

#[wasm_bindgen]
pub fn apply_features(features: JsValue) -> JsResult<Uint8Array> {
    let features: WasmEarsFeatures = serde_wasm_bindgen::from_value(features)?;
    
    panic!("features: {:?}", features);
}