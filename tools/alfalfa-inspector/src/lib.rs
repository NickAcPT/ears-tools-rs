/* export type AlfalfaPredefinedKey = "END" | "wing" | "erase" | "cape";
export type AlfalfaKey = AlfalfaPredefinedKey | string;

export interface AlfalfaBinaryEntryData {
    type: "binary";
    value: Uint8Array;
}

export interface AlfalfaImageEntryData {
    type: "image";
    value: Uint8Array;
}

export interface AlfalfaEraseEntryData {
    type: "erase";
    value: {
        x: number;
        y: number;
        width: number;
        height: number;
    }[];
}

export type AlfalfaEntryData = AlfalfaBinaryEntryData | AlfalfaImageEntryData | AlfalfaEraseEntryData;

export type AlfalfaDataObject = Record<AlfalfaKey, AlfalfaEntryData>;
 */

use ears_rs::alfalfa::{utils::{EraseRegionsProvider, EraseRegion}, AlfalfaDataKey, AlfalfaData};
use image::ImageFormat;
use js_sys::{Object, Reflect, Uint8Array};
use js_utils::JsResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Cursor, mem::transmute};
use wasm_bindgen::prelude::*;

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

fn serialize_alfalfa_data_map(data: AlfalfaDataMap) -> JsResult<JsValue> {
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

fn deserialize_alfalfa_data_map(data: JsValue) -> JsResult<AlfalfaDataMap> {
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

pub struct AlfalfaDataValueGetter {
    _version: u8,
    data: HashMap<String, Vec<u8>>,
}

#[wasm_bindgen]
pub fn read_alfalfa_data(data: &[u8]) -> JsResult<JsValue> {
    console_error_panic_hook::set_once();

    let skin = image::load_from_memory(data)?.into_rgba8();
    let mut map = HashMap::new();

    let alfalfa = ears_rs::alfalfa::read_alfalfa(&skin)?;

    if let Some(alfalfa) = alfalfa {
        let alfalfa_getter: AlfalfaDataValueGetter = unsafe { transmute(alfalfa.clone()) };

        for (key, value) in alfalfa_getter.data {
            let entry_data = if &key == Into::<&'static str>::into(AlfalfaDataKey::Cape)
                || &key == Into::<&'static str>::into(AlfalfaDataKey::Wings)
            {
                AlfalfaEntryData::Image(value)
            } else if &key == Into::<&'static str>::into(AlfalfaDataKey::Erase) {
                let regions = alfalfa
                    .get_erase_regions()?
                    .unwrap_or_default()
                    .into_iter()
                    .map(|region| AlfalfaEraseEntryData {
                        x: region.x,
                        y: region.y,
                        width: region.width,
                        height: region.height,
                    })
                    .collect();

                AlfalfaEntryData::Erase(regions)
            } else {
                AlfalfaEntryData::Binary(value)
            };

            map.insert(key, entry_data);
        }
    }

    Ok(serialize_alfalfa_data_map(map)?)
}

#[wasm_bindgen]
pub fn write_alfalfa_data(image_data: &[u8], workspace: JsValue) -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();

    let map = deserialize_alfalfa_data_map(workspace)?;
    let mut alfalfa = AlfalfaData::new();
    
    for (key, data) in map {
        if &key == Into::<&'static str>::into(AlfalfaDataKey::Erase) {
            if let AlfalfaEntryData::Erase(data) = data {
                let mut regions = Vec::new();

                for region in data {
                    regions.push(EraseRegion {
                        x: region.x,
                        y: region.y,
                        width: region.width,
                        height: region.height,
                    });
                }

                alfalfa.set_erase_regions(&regions)?;
            }
        } else if let AlfalfaEntryData::Binary(data) | AlfalfaEntryData::Image(data) = data {
            let key = Box::leak(key.into_boxed_str());
            
            alfalfa.set_data(AlfalfaDataKey::Custom(key), data);
        }
    }
    
    let mut skin = image::load_from_memory(image_data)?.into_rgba8();

    ears_rs::alfalfa::write_alfalfa(&alfalfa, &mut skin)?;

    let mut bytes = Vec::new();
    {
        skin.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    }

    Ok(bytes.as_slice().into())
}
