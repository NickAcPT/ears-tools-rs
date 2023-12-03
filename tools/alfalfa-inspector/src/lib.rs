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

use ears_rs::alfalfa::{utils::{EraseRegionsProvider, EraseRegion}, AlfalfaDataKey};
use image::ImageFormat;
use js_utils::JsResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, mem::transmute, io::Cursor};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::{Uint8Array, Reflect};

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

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AlfalfaEntryDataSerialized {
    #[serde(rename = "binary")]
    Binary {
        #[serde(with = "serde_wasm_bindgen::preserve")]
        value: Uint8Array,
    },
    #[serde(rename = "image")]
    Image {
        #[serde(with = "serde_wasm_bindgen::preserve")]
        value: Uint8Array,
    },
    #[serde(rename = "erase")]
    Erase {
        #[serde(with = "serde_wasm_bindgen::preserve")]
        value: JsValue,
    },
}

impl AlfalfaEntryDataSerialized {
    fn from(data: AlfalfaEntryData) -> JsResult<Self> {
        match data {
            AlfalfaEntryData::Binary(value) => Ok(Self::Binary{
                value: value.as_slice().into()
            }),
            AlfalfaEntryData::Image(value) => Ok(Self::Image{
                value: value.as_slice().into()
            }),
            AlfalfaEntryData::Erase(value) => Ok(Self::Erase {
                value: value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?,
            }),
        }
    }
    
    fn deserialize(&self) -> JsResult<AlfalfaEntryData> {
        match self {
            Self::Binary { value } => Ok(AlfalfaEntryData::Binary(value.to_vec())),
            Self::Image { value } => Ok(AlfalfaEntryData::Image(value.to_vec())),
            Self::Erase { value } => {
                let value: Vec<AlfalfaEraseEntryData> =
                    serde_wasm_bindgen::from_value(value.clone())?;

                Ok(AlfalfaEntryData::Erase(value))
            }
        }
    }
}

pub type AlfalfaDataMap = HashMap<String, AlfalfaEntryData>;
pub type AlfalfaDataMapSerialized = HashMap<String, AlfalfaEntryDataSerialized>;

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

            map.insert(key, AlfalfaEntryDataSerialized::from(entry_data)?);
        }
    }

    let json_compatible = serde_wasm_bindgen::Serializer::json_compatible();
    Ok(map.serialize(&json_compatible)?.into())
}


#[wasm_bindgen]
pub fn write_alfalfa_data(image_data: &[u8], data: JsValue) -> JsResult<Uint8Array> {
    console_error_panic_hook::set_once();
    
    let mut skin = image::load_from_memory(image_data)?.into_rgba8();
    
    ears_rs::alfalfa::write_alfalfa(todo!("&alfalfa"), &mut skin)?;

    let mut bytes = Vec::new();
    {
        skin.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    }
    
    Ok(bytes.as_slice().into())
}