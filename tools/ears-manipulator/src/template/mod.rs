use std::iter::repeat;

use ears_rs::parser::EarsParser;
use image::RgbaImage;
use js_utils::JsResult;
use nmsr_player_parts::{
    parts::provider::{PartsProvider, PlayerPartProviderContext, PlayerPartsProvider},
    types::PlayerBodyPartType,
};
use strum::IntoEnumIterator;

use crate::{
    model::{
        WasmEarsEmissiveData, WasmEarsFeatures, WasmEarsSettings, WasmTailSettings,
        WasmWingSettings,
    },
    template::model::apply_part_template,
};

#[allow(dead_code)]
mod model;

pub(crate) fn apply_template(
    skin_image: &mut RgbaImage,
    wasm_features: &WasmEarsFeatures,
    apply_only_minecraft_parts: bool,
    slim_arms: bool,
) -> JsResult<()> {
    let features = WasmEarsFeatures {
        ears: WasmEarsSettings {
            anchor: wasm_features.ears.anchor,
            mode: Some(wasm_features.ears.mode)
                .filter(|_| wasm_features.ears.source.is_sample_skin())
                .unwrap_or_default(),
            source: wasm_features.ears.source,
        },
        protrusions: if wasm_features.protrusions_source.is_sample_skin() {
            wasm_features.protrusions.clone()
        } else {
            Vec::with_capacity(0)
        },
        protrusions_source: wasm_features.protrusions_source,
        leg_mode: wasm_features.leg_mode,
        tail: WasmTailSettings {
            mode: Some(wasm_features.tail.mode)
                .filter(|_| wasm_features.tail.source.is_sample_skin())
                .unwrap_or_default(),
            segments: 1,
            bends: [0.0; 4],
            animate: wasm_features.tail.animate,
            swap_jacket_back: wasm_features.tail.swap_jacket_back,
            source: wasm_features.tail.source,
        },
        snout: if wasm_features
            .snout
            .is_some_and(|s| s.source.is_sample_skin())
        {
            wasm_features.snout.clone()
        } else {
            None
        },
        wings: WasmWingSettings {
            mode: Some(wasm_features.wings.mode)
                .filter(|_| wasm_features.wings.source.is_sample_skin())
                .unwrap_or_default(),
            source: wasm_features.wings.source,
            animations: wasm_features.wings.animations,
            wings: None,
        },
        cape: None,
        locator: None,
        chest_size: wasm_features.chest_size,
        alfalfa: None,
        emissives: WasmEarsEmissiveData {
            enabled: false,
            palette: Vec::with_capacity(0),
        },
        data_version: 1,
        apply_template: wasm_features.apply_template,
    };

    let context: PlayerPartProviderContext<()> = PlayerPartProviderContext {
        model: if slim_arms {
            nmsr_player_parts::model::PlayerModel::Alex
        } else {
            nmsr_player_parts::model::PlayerModel::Steve
        },
        has_hat_layer: true,
        has_layers: true,
        has_deadmau5_ears: false,
        is_flipped_upside_down: false,
        has_cape: false,
        custom_arm_rotation_z: Some(0f32),
        shadow_y_pos: None,
        shadow_is_square: false,
        armor_slots: None,
        ears_features: Some(features.into()),
        movement: Default::default(),
    };

    let parts = if apply_only_minecraft_parts {
        [PlayerPartsProvider::Minecraft]
    } else {
        [PlayerPartsProvider::Ears]
    };
    let parts = parts
        .iter()
        .flat_map(|provider| {
            PlayerBodyPartType::iter()
                .flat_map(|p| provider.get_parts(&context, p).into_iter().zip(repeat(p)))
        })
        .collect::<Vec<_>>();

    for (part, body_part) in parts {
        apply_part_template(body_part, &part, skin_image);
    }

    Ok(())
}
