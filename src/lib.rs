use std::io::Cursor;

use ears_rs::alfalfa::{
    utils::{EraseRegion, EraseRegionsProvider},
    AlfalfaData,
};
use image::ImageFormat;
use wasm_bindgen::prelude::*;

extern crate alloc;

use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};

// SAFETY: This application is single threaded, so using AssumeSingleThreaded is allowed.
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

#[wasm_bindgen]
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct WasmEraseRegion {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
}

impl From<WasmEraseRegion> for EraseRegion {
    fn from(value: WasmEraseRegion) -> EraseRegion {
        EraseRegion {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

impl From<EraseRegion> for WasmEraseRegion {
    fn from(value: EraseRegion) -> WasmEraseRegion {
        WasmEraseRegion {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

#[wasm_bindgen]
pub struct EarsImageWorkspace {
    alfalfa: AlfalfaData,
    regions: Vec<WasmEraseRegion>,
}

#[wasm_bindgen]
pub fn decode_ears_image(skin_bytes: &[u8]) -> EarsImageWorkspace {
    let image = image::load_from_memory_with_format(skin_bytes, ImageFormat::Png)
        .unwrap_throw()
        .into_rgba8();

    let data = ears_rs::alfalfa::read_alfalfa(&image).unwrap_throw();
    let data = data.unwrap_or_else(|| AlfalfaData::new());

    let regions = data
        .get_erase_regions()
        .unwrap_throw()
        .unwrap_or_else(|| vec![])
        .into_iter()
        .map(|r| r.into())
        .collect();

    EarsImageWorkspace {
        alfalfa: data,
        regions,
    }
}

#[wasm_bindgen]
pub fn encode_ears_image(skin_bytes: &[u8], workspace: EarsImageWorkspace) -> Vec<u8> {
    let mut image = image::load_from_memory_with_format(skin_bytes, ImageFormat::Png)
        .unwrap_throw()
        .into_rgba8();

    let mut data = workspace.alfalfa;

    data.set_erase_regions(workspace.regions.iter().map(|r| (*r).into()).collect())
        .unwrap_throw();

    ears_rs::alfalfa::write_alfalfa(&data, &mut image).unwrap_throw();

    if data.is_empty() {
        ears_rs::utils::strip_alpha(&mut image);
    }

    let mut bytes = Vec::new();
    {
        image
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap_throw();
    }

    bytes
}

#[wasm_bindgen]
pub fn get_regions(decoded: &EarsImageWorkspace) -> JsValue {
    serde_wasm_bindgen::to_value(&decoded.regions).unwrap_throw()
}

#[wasm_bindgen]
pub fn set_regions(decoded: &mut EarsImageWorkspace, regions: JsValue) {
    decoded.regions = serde_wasm_bindgen::from_value(regions).unwrap_throw();
}
