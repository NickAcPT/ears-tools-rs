use ears_rs::alfalfa::{utils::EraseRegion, AlfalfaData};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{models::WasmEraseRegion, ResultExt};

#[wasm_bindgen]
pub struct EarsImageWorkspace {
    pub(crate) alfalfa: AlfalfaData,
    pub(crate) regions: Vec<EraseRegion>,
}

#[wasm_bindgen]
impl EarsImageWorkspace {
    pub fn get_regions(&self) -> Result<JsValue, JsValue> {
        let wasm_regions: Vec<WasmEraseRegion> =
            self.regions.iter().map(|r| (*r).into()).collect::<Vec<_>>();

        serde_wasm_bindgen::to_value(&wasm_regions).into_js()
    }

    pub fn set_regions(&mut self, regions: JsValue) -> Result<(), JsValue> {
        let wasm_regions: Vec<WasmEraseRegion> =
            serde_wasm_bindgen::from_value(regions).into_js()?;
        let ears_regions: Vec<EraseRegion> = wasm_regions
            .into_iter()
            .map(|r| r.into())
            .collect::<Vec<_>>();

        self.regions = ears_regions;
        Ok(())
    }
}
