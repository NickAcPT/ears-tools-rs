use crate::models::EarsImageWorkspace;
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};
use std::fmt::Display;
use wasm_bindgen::prelude::*;

extern crate alloc;

pub mod errors;
mod logic;
mod models;

// SAFETY: This application is single threaded, so using AssumeSingleThreaded is allowed.
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

#[wasm_bindgen]
pub fn decode_ears_image(skin_bytes: &[u8]) -> Result<EarsImageWorkspace, JsValue> {
    Ok(logic::decode_ears_image(skin_bytes).into_js()?.into())
}

#[wasm_bindgen]
pub fn encode_ears_image(
    skin_bytes: &[u8],
    workspace: &mut EarsImageWorkspace,
) -> Result<Vec<u8>, JsValue> {
    Ok(logic::encode_ears_image(skin_bytes, workspace).into_js()?)
}

trait ResultExt<T, E> {
    fn into_js(self) -> Result<T, JsValue>;
}

impl<T, E: Display> ResultExt<T, E> for Result<T, E> {
    fn into_js(self) -> Result<T, JsValue> {
        self.map_err(|e| e.to_string().into())
    }
}
