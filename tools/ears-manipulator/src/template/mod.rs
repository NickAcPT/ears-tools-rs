use std::iter::repeat;

use ears_rs::features::{
    data::wing::{WingAnimationMode, WingData, WingMode},
    EarsFeatures,
};
use image::{imageops, RgbaImage};
use js_utils::JsResult;
use nmsr_player_parts::{
    parts::provider::{
        ears::PlayerPartEarsTextureType, PartsProvider, PlayerPartProviderContext,
        PlayerPartsProvider,
    },
    types::{PlayerBodyPartType, PlayerPartTextureType},
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

    let provider = if apply_only_minecraft_parts {
        PlayerPartsProvider::Minecraft
    } else {
        PlayerPartsProvider::Ears
    };

    paint_template(
        provider,
        &template_context(slim_arms, Some(features.into()), false),
        PlayerPartTextureType::Skin,
        skin_image,
    );

    Ok(())
}

/// Sample wings texture, laid out for `mode` (asymmetric wings use one half per wing).
pub(crate) fn wings_template(mode: WingMode) -> RgbaImage {
    let mode = if mode == WingMode::None {
        WingMode::SymmetricDual
    } else {
        mode
    };

    let features = EarsFeatures {
        wing: Some(WingData {
            mode,
            animation_mode: WingAnimationMode::None,
        }),
        ..Default::default()
    };

    let texture: PlayerPartTextureType = PlayerPartEarsTextureType::Wings.into();
    let (width, height) = texture.get_texture_size();
    let mut image = RgbaImage::new(width, height);

    paint_template(
        PlayerPartsProvider::Ears,
        &template_context(false, Some(features), false),
        texture,
        &mut image,
    );

    image
}

/// Sample cape texture in the 20x16 Ears layout: the outside of the cape on the left half,
/// the inside on the right half.
pub(crate) fn cape_template() -> RgbaImage {
    let texture = PlayerPartTextureType::Cape;
    let (width, height) = texture.get_texture_size();
    let mut mojang_cape = RgbaImage::new(width, height);

    paint_template(
        PlayerPartsProvider::Minecraft,
        &template_context(false, None, true),
        texture,
        &mut mojang_cape,
    );

    // Inverse of `ears_rs::utils::convert_ears_cape_to_mojang_cape`; the edges are derived from these faces.
    let mut cape = RgbaImage::new(20, 16);
    imageops::replace(&mut cape, &*imageops::crop_imm(&mojang_cape, 1, 1, 10, 16), 0, 0);
    imageops::replace(&mut cape, &*imageops::crop_imm(&mojang_cape, 12, 1, 10, 16), 10, 0);

    cape
}

fn template_context(
    slim_arms: bool,
    ears_features: Option<EarsFeatures>,
    has_cape: bool,
) -> PlayerPartProviderContext<()> {
    PlayerPartProviderContext {
        model: if slim_arms {
            nmsr_player_parts::model::PlayerModel::Alex
        } else {
            nmsr_player_parts::model::PlayerModel::Steve
        },
        has_hat_layer: true,
        has_layers: true,
        has_deadmau5_ears: false,
        is_flipped_upside_down: false,
        has_cape,
        custom_arm_rotation_z: Some(0f32),
        shadow_y_pos: None,
        shadow_is_square: false,
        armor_slots: None,
        ears_features,
        movement: Default::default(),
    }
}

/// Paints the template of every part `provider` draws with `texture` onto `image`.
fn paint_template(
    provider: PlayerPartsProvider,
    context: &PlayerPartProviderContext<()>,
    texture: PlayerPartTextureType,
    image: &mut RgbaImage,
) {
    let parts = PlayerBodyPartType::iter()
        .flat_map(|p| provider.get_parts(context, p).into_iter().zip(repeat(p)));

    for (part, body_part) in parts {
        apply_part_template(body_part, &part, texture, image);
    }
}
