use std::collections::HashMap;

use js_sys::{Object, Reflect};
use js_utils::JsResult;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsValue, JsError};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AlfalfaEraseEntryData {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AlfalfaEntryData {
    #[serde(rename = "binary", with = "serde_bytes")]
    Binary(Vec<u8>),
    #[serde(rename = "image", with = "serde_bytes")]
    Image(Vec<u8>),
    #[serde(rename = "erase")]
    Erase(Vec<AlfalfaEraseEntryData>),
}

pub type AlfalfaDataMap = HashMap<String, AlfalfaEntryData>;

pub(crate) fn serialize_alfalfa_data_map(data: AlfalfaDataMap) -> JsResult<JsValue> {
    fn serialize_alfalfa_entry_data(data: AlfalfaEntryData) -> JsResult<JsValue> {
        Ok(data.serialize(&serde_wasm_bindgen::Serializer::default())?)
    }

    let obj = Object::new();

    for (key, data) in data {
        Reflect::set(&obj, &key.into(), &serialize_alfalfa_entry_data(data)?).map_err(|_| JsError::new("Failed to set property 2"))?;
    }

    Ok(obj.into())
}

pub(crate) fn deserialize_alfalfa_data_map(data: JsValue) -> JsResult<AlfalfaDataMap> {
    fn deserialize_alfalfa_entry_data(data: JsValue) -> JsResult<AlfalfaEntryData> {
        let data = serde_wasm_bindgen::from_value(data).map_err(|_| JsError::new("Failed to deserialize data"))?;
        
        Ok(data)
    }

    let data = Object::from(data);
    let mut map = HashMap::new();

    for js_key in Reflect::own_keys(&data)
        .map_err(|_| JsError::new("Failed to get keys"))?
        .iter()
    {
        let key = js_key.as_string().ok_or_else(|| JsError::new("Invalid key"))?;
        let value =
            Reflect::get(&data, &js_key).map_err(|_| JsError::new("Failed to get value"))?;

        map.insert(key, deserialize_alfalfa_entry_data(value)?);
    }

    Ok(map)
}