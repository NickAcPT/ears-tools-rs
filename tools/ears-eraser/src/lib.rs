use crate::models::EarsImageWorkspace;
use js_utils::JsResult;
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};
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
pub fn decode_ears_image(skin_bytes: &[u8]) -> JsResult<EarsImageWorkspace> {
    Ok(logic::decode_ears_image(skin_bytes)?.into())
}

#[wasm_bindgen]
pub fn encode_ears_image(
    skin_bytes: &[u8],
    workspace: &mut EarsImageWorkspace,
) -> JsResult<Vec<u8>> {
    Ok(logic::encode_ears_image(skin_bytes, workspace)?)
}
