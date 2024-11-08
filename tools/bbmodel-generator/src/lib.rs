use std::ops::Deref;

use js_utils::JsResult;
use nmsr_rendering_blockbench_model_generator_experiment::{
    blockbench,
    generator::{new_model_generator_without_part_context, DefaultImageIO},
    nmsr_rendering::high_level::{model::PlayerModel, types::PlayerPartTextureType},
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue, UnwrapThrowExt};

extern crate alloc;

#[cfg(target_arch = "wasm32")]
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};
use web_sys::console::log_1;

// SAFETY: This application is single threaded, so using AssumeSingleThreaded is allowed.
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

#[wasm_bindgen]
pub enum WasmPlayerModel {
    Steve,
    Alex,
}

impl Deref for WasmPlayerModel {
    type Target = PlayerModel;

    fn deref(&self) -> &Self::Target {
        match self {
            WasmPlayerModel::Steve => &PlayerModel::Steve,
            WasmPlayerModel::Alex => &PlayerModel::Alex,
        }
    }
}

#[wasm_bindgen]
pub fn generate_blockbench_model(
    skin_bytes: &[u8],
    model: WasmPlayerModel,
    layers: bool,
) -> JsResult<JsValue> {
    let mut project = new_model_generator_without_part_context(*model, layers, DefaultImageIO);

    project
        .load_texture(PlayerPartTextureType::Skin, &skin_bytes, true)
        .unwrap_throw();

    for warning in project.warnings().iter() {
        log_1(&JsValue::from_str(&warning));

        if let Some(window) = web_sys::window() {
            window.alert_with_message(&warning).unwrap_throw();
        }
    }

    let result = blockbench::generate_project(project);

    Ok(result?)
}
