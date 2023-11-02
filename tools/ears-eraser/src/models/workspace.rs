use ears_rs::alfalfa::{utils::EraseRegion, AlfalfaData};
use js_utils::JsResult;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::models::WasmEraseRegion;

#[wasm_bindgen]
pub struct EarsImageWorkspace {
    pub(crate) alfalfa: AlfalfaData,
    pub(crate) regions: Vec<EraseRegion>,
}

#[wasm_bindgen]
impl EarsImageWorkspace {
    pub fn get_regions(&self) -> JsResult<JsValue> {
        let wasm_regions: Vec<WasmEraseRegion> =
            self.regions.iter().map(|r| (*r).into()).collect::<Vec<_>>();

        Ok(serde_wasm_bindgen::to_value(&wasm_regions)?)
    }

    pub fn set_regions(&mut self, regions: JsValue) -> JsResult<()> {
        let wasm_regions: Vec<WasmEraseRegion> = serde_wasm_bindgen::from_value(regions)?;
        let ears_regions: Vec<EraseRegion> = wasm_regions
            .into_iter()
            .map(|r| r.into())
            .collect::<Vec<_>>();

        self.regions = ears_regions;

        Ok(())
    }
}
