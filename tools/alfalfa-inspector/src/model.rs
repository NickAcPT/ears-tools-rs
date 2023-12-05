use std::collections::HashMap;

use js_sys::{Object, Reflect, Uint8Array};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlfalfaEntryData {
    Binary(Vec<u8>),
    Image(Vec<u8>),
    Erase(Vec<AlfalfaEraseEntryData>),
}

pub type AlfalfaDataMap = HashMap<String, AlfalfaEntryData>;

pub(crate) fn serialize_alfalfa_data_map(data: AlfalfaDataMap) -> JsResult<JsValue> {
    fn serialize_alfalfa_entry_data(data: AlfalfaEntryData) -> JsResult<JsValue> {
        let obj = Object::new();

        let tag = match data {
            AlfalfaEntryData::Binary(_) => "binary",
            AlfalfaEntryData::Image(_) => "image",
            AlfalfaEntryData::Erase(_) => "erase",
        };

        Reflect::set(&obj, &"type".into(), &tag.into()).map_err(|_| JsError::new("Failed to set type"))?;

        let data: JsValue = match data {
            AlfalfaEntryData::Binary(data) | AlfalfaEntryData::Image(data) => {
                Uint8Array::from(&data[..]).into()
            }
            AlfalfaEntryData::Erase(data) => {
                let array = js_sys::Array::new();

                for region in data {
                    let obj = Object::new();

                    Reflect::set(&obj, &"x".into(), &region.x.into()).map_err(|_| JsError::new("Failed to set property x"))?;
                    Reflect::set(&obj, &"y".into(), &region.y.into()).map_err(|_| JsError::new("Failed to set property y"))?;
                    Reflect::set(&obj, &"width".into(), &region.width.into()).map_err(|_| JsError::new("Failed to set property width"))?;
                    Reflect::set(&obj, &"height".into(), &region.height.into()).map_err(|_| JsError::new("Failed to set property height"))?;

                    array.push(&obj);
                }

                array.into()
            }
        };

        Reflect::set(&obj, &"value".into(), &data).map_err(|_| JsError::new("Failed to set property 1"))?;

        Ok(obj.into())
    }

    let obj = Object::new();

    for (key, data) in data {
        Reflect::set(&obj, &key.into(), &serialize_alfalfa_entry_data(data)?).map_err(|_| JsError::new("Failed to set property 2"))?;
    }

    Ok(obj.into())
}

pub(crate) fn deserialize_alfalfa_data_map(data: JsValue) -> JsResult<AlfalfaDataMap> {
    fn deserialize_alfalfa_entry_data(data: JsValue) -> JsResult<AlfalfaEntryData> {
        let data = Object::from(data);
        let tag =
            Reflect::get(&data, &"type".into()).map_err(|_| JsError::new("Failed to get type"))?;

        let tag = tag
            .as_string()
            .ok_or_else(|| JsError::new("Invalid type tag"))?;
        let data = Reflect::get(&data, &"value".into())
            .map_err(|_| JsError::new("Failed to get value"))?;

        let data = match tag.as_str() {
            "binary" => {
                let data = Uint8Array::from(data);

                AlfalfaEntryData::Binary(data.to_vec())
            }
            "image" => {
                let data = Uint8Array::from(data);

                AlfalfaEntryData::Image(data.to_vec())
            }
            "erase" => {
                let data = js_sys::Array::from(&data);

                let mut regions = Vec::new();

                for region in data.iter() {
                    let region = Object::from(region);

                    let x = Reflect::get(&region, &"x".into())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let y = Reflect::get(&region, &"y".into())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let width = Reflect::get(&region, &"width".into())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let height = Reflect::get(&region, &"height".into())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);

                    let x = x as u8;
                    let y = y as u8;
                    let width = width as u8;
                    let height = height as u8;

                    regions.push(AlfalfaEraseEntryData {
                        x,
                        y,
                        width,
                        height,
                    });
                }

                AlfalfaEntryData::Erase(regions)
            }
            _ => return Err(JsError::new("Invalid type tag").into()),
        };

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