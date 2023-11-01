use ears_rs::alfalfa::utils::EraseRegion;
use wasm_bindgen::prelude::wasm_bindgen;

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