use ears_rs::alfalfa::{
    read_alfalfa,
    utils::{EraseRegion, EraseRegionsProvider},
    AlfalfaData, AlfalfaDataKey,
};
use image::ImageFormat;
use js_sys::Uint8Array;
use js_utils::JsResult;
use std::{collections::HashMap, io::Cursor};
use wasm_bindgen::prelude::*;

mod model;

use model::*;

#[cfg(target_arch = "wasm32")]
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};

// SAFETY: This application is single threaded, so using AssumeSingleThreaded is allowed.
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

#[wasm_bindgen]
pub fn read_alfalfa_data(data: &[u8]) -> JsResult<JsValue> {
    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
    }

    let skin = image::load_from_memory(data)?.into_rgba8();
    let mut map = HashMap::new();

    let alfalfa = read_alfalfa(&skin)?;

    if let Some(alfalfa_data) = alfalfa {
        for (key, value) in alfalfa_data.get_data_raw() {
            let key = key.to_owned();
            let value = value.to_owned();

            let entry_data = if key == Into::<&'static str>::into(AlfalfaDataKey::Cape)
                || key == Into::<&'static str>::into(AlfalfaDataKey::Wings)
            {
                AlfalfaEntryData::Image(value)
            } else if key == Into::<&'static str>::into(AlfalfaDataKey::Erase) {
                let regions = alfalfa_data
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
