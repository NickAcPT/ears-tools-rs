use std::iter::repeat;

use ears_rs::parser::EarsParser;
use image::RgbaImage;
use js_utils::JsResult;
use nmsr_player_parts::{
    parts::provider::{PartsProvider, PlayerPartProviderContext, PlayerPartsProvider},
    types::PlayerBodyPartType,
};
use strum::IntoEnumIterator;

use crate::{model::{WasmEarsEmissiveData, WasmEarsFeatures, WasmEarsSettings, WasmTailSettings, WasmWingSettings}, template::model::PartTemplateGeneratorContext};

#[allow(dead_code)]
mod model;

static mut PART_TEMPLATE_CONTEXT: Option<PartTemplateGeneratorContext> = None;

pub(crate) fn apply_template(
    skin_image: &mut RgbaImage,
    wasm_features: &WasmEarsFeatures,
) -> JsResult<()> {
    if let None = unsafe { PART_TEMPLATE_CONTEXT.as_ref() } {
        unsafe {
            PART_TEMPLATE_CONTEXT = Some(PartTemplateGeneratorContext::new());
        }
    }
    let part_template_context = unsafe { PART_TEMPLATE_CONTEXT.as_ref().unwrap() };
    
    let features = WasmEarsFeatures {
        ears: WasmEarsSettings {
            anchor: wasm_features.ears.anchor,
            mode: Some(wasm_features.ears.mode).filter(|_| wasm_features.ears.source.is_sample_skin()).unwrap_or_default(),
            source: wasm_features.ears.source,
        },
        protrusions: if wasm_features.protrusions_source.is_sample_skin() {
            wasm_features.protrusions.clone()
        } else {
            Vec::with_capacity(0)
        },
        protrusions_source: wasm_features.protrusions_source,
        tail: WasmTailSettings {
            mode: Some(wasm_features.tail.mode).filter(|_| wasm_features.tail.source.is_sample_skin()).unwrap_or_default(),
            segments: 1,
            bends: [0.0; 4],
            source: wasm_features.tail.source,
        },
        snout: if wasm_features.snout.is_some_and(|s| s.source.is_sample_skin()) {
            wasm_features.snout.clone()
        } else {
            None
        },
        wings: WasmWingSettings {
            mode: Some(wasm_features.wings.mode).filter(|_| wasm_features.wings.source.is_sample_skin()).unwrap_or_default(),
            source: wasm_features.wings.source,
            animations: wasm_features.wings.animations,
            wings: None
        },
        cape: None,
        chest_size: wasm_features.chest_size,
        alfalfa: None,
        emissives: WasmEarsEmissiveData {
            enabled: false,
            palette: Vec::with_capacity(0)
        },
        data_version: 1,
    };

    let context: PlayerPartProviderContext<()> = PlayerPartProviderContext {
        model: nmsr_player_parts::model::PlayerModel::Alex,
        has_hat_layer: true,
        has_layers: true,
        has_deadmau5_ears: false,
        is_flipped_upside_down: false,
        has_cape: false,
        arm_rotation: 0f32,
        shadow_y_pos: None,
        shadow_is_square: false,
        armor_slots: None,
        ears_features: Some(features.into()),
    };
    
    let parts = [PlayerPartsProvider::Ears]
        .iter()
        .flat_map(|provider| {
            PlayerBodyPartType::iter()
                .flat_map(|p| provider.get_parts(&context, p).into_iter().zip(repeat(p)))
        })
        .collect::<Vec<_>>();

        
    for (part, body_part) in parts {
        part_template_context.handle_part_texture(body_part, part, skin_image);
    }
    
    Ok(())
}
