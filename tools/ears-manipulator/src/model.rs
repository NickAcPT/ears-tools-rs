/* export enum TextureSource {
    SampleSkin,
    YourSkin,
}

export interface WasmEarsFeatures {
    ears: EarsSettings;
    protrusions: Set<Protrusion>;
    tail: TailSettings;
    snout?: SnoutSettings;
    wings: WingSettings;
    cape?: Uint8Array;
    chestSize: number;
    alfalfa?: WasmAlfalfaData;
}

export interface WasmAlfalfaData {
    version: number;
    data: Record<string, Uint8Array>;
}

export interface WasmEarsSettings {
    mode: EarsMode;
    anchor: EarsAnchor;
    source: TextureSource;
}

export interface WasmTailSettings {
    mode: TailMode;
    segments: 1 | 2 | 3 | 4;
    bends: number[];
    source: TextureSource;
}

export enum WasmSnoutStatus {
    Disabled,
    Enabled,
}

export interface WasmSnoutSettings {
    width: number;
    height: number;
    length: number;
    offset: number;
    source: TextureSource;
}

export interface WasmWingSettings {
    mode: WingsMode;
    animations: WingsAnimations;
    wings: Uint8Array;
    source: TextureSource;
}

export enum WasmEarsMode {
    None,
    Above,
    Sides,
    Behind,
    Around,
    Floppy,
    Out,
    Cross,
    Tall,
    TallCross
}

export enum WasmEarsAnchor {
    Center,
    Front,
    Back
}

export enum WasmProtrusion {
    Claws,
    Horns
}

export enum WasmTailMode {
    None,
    Down,
    Back,
    Up,
    Vertical
}

export enum WasmWingsMode {
    None,
    SymmetricDual,
    SymmetricSingle,
    AsymmetricSingleLeft,
    AsymmetricSingleRight,
}

export enum WasmWingsAnimations {
    Normal,
    None
} */

use std::collections::HashMap;

use ears_rs::{
    alfalfa::{AlfalfaData, AlfalfaDataKey},
    features::{
        data::{
            ear::{EarAnchor, EarMode},
            snout::SnoutData,
            tail::{TailData, TailMode},
            wing::{WingData, WingMode},
        },
        EarsFeatures,
    }, utils::EarsEmissivePalette,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum WasmEarsMode {
    None,
    Above,
    Sides,
    Behind,
    Around,
    Floppy,
    Out,
    Cross,
    Tall,
    TallCross,
}

impl From<WasmEarsMode> for EarMode {
    fn from(mode: WasmEarsMode) -> Self {
        match mode {
            WasmEarsMode::None => EarMode::None,
            WasmEarsMode::Above => EarMode::Above,
            WasmEarsMode::Sides => EarMode::Sides,
            WasmEarsMode::Behind => EarMode::Behind,
            WasmEarsMode::Around => EarMode::Around,
            WasmEarsMode::Floppy => EarMode::Floppy,
            WasmEarsMode::Out => EarMode::Out,
            WasmEarsMode::Cross => EarMode::Cross,
            WasmEarsMode::Tall => EarMode::Tall,
            WasmEarsMode::TallCross => EarMode::TallCross,
        }
    }
}

impl From<EarMode> for WasmEarsMode {
    fn from(mode: EarMode) -> Self {
        match mode {
            EarMode::None => WasmEarsMode::None,
            EarMode::Above => WasmEarsMode::Above,
            EarMode::Sides => WasmEarsMode::Sides,
            EarMode::Behind => WasmEarsMode::Behind,
            EarMode::Around => WasmEarsMode::Around,
            EarMode::Floppy => WasmEarsMode::Floppy,
            EarMode::Out => WasmEarsMode::Out,
            EarMode::Cross => WasmEarsMode::Cross,
            EarMode::Tall => WasmEarsMode::Tall,
            EarMode::TallCross => WasmEarsMode::TallCross,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub(crate) enum WasmEarsAnchor {
    Center,
    Front,
    Back,
}

impl From<WasmEarsAnchor> for EarAnchor {
    fn from(anchor: WasmEarsAnchor) -> Self {
        match anchor {
            WasmEarsAnchor::Center => EarAnchor::Center,
            WasmEarsAnchor::Front => EarAnchor::Front,
            WasmEarsAnchor::Back => EarAnchor::Back,
        }
    }
}

impl From<EarAnchor> for WasmEarsAnchor {
    fn from(anchor: EarAnchor) -> Self {
        match anchor {
            EarAnchor::Center => WasmEarsAnchor::Center,
            EarAnchor::Front => WasmEarsAnchor::Front,
            EarAnchor::Back => WasmEarsAnchor::Back,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub(crate) enum WasmProtrusion {
    Claws,
    Horns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub(crate) enum WasmTailMode {
    None,
    Down,
    Back,
    Up,
    Vertical,
}

impl From<WasmTailMode> for TailMode {
    fn from(mode: WasmTailMode) -> Self {
        match mode {
            WasmTailMode::None => TailMode::None,
            WasmTailMode::Down => TailMode::Down,
            WasmTailMode::Back => TailMode::Back,
            WasmTailMode::Up => TailMode::Up,
            WasmTailMode::Vertical => TailMode::Vertical,
        }
    }
}

impl From<TailMode> for WasmTailMode {
    fn from(mode: TailMode) -> Self {
        match mode {
            TailMode::None => WasmTailMode::None,
            TailMode::Down => WasmTailMode::Down,
            TailMode::Back => WasmTailMode::Back,
            TailMode::Up => WasmTailMode::Up,
            TailMode::Vertical => WasmTailMode::Vertical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub(crate) enum WasmWingsMode {
    None,
    SymmetricDual,
    SymmetricSingle,
    AsymmetricSingleLeft,
    AsymmetricSingleRight,
}

impl From<WasmWingsMode> for WingMode {
    fn from(mode: WasmWingsMode) -> Self {
        match mode {
            WasmWingsMode::None => WingMode::None,
            WasmWingsMode::SymmetricDual => WingMode::SymmetricDual,
            WasmWingsMode::SymmetricSingle => WingMode::SymmetricSingle,
            WasmWingsMode::AsymmetricSingleLeft => WingMode::AsymmetricL,
            WasmWingsMode::AsymmetricSingleRight => WingMode::AsymmetricR,
        }
    }
}

impl From<WingMode> for WasmWingsMode {
    fn from(mode: WingMode) -> Self {
        match mode {
            WingMode::None => WasmWingsMode::None,
            WingMode::SymmetricDual => WasmWingsMode::SymmetricDual,
            WingMode::SymmetricSingle => WasmWingsMode::SymmetricSingle,
            WingMode::AsymmetricL => WasmWingsMode::AsymmetricSingleLeft,
            WingMode::AsymmetricR => WasmWingsMode::AsymmetricSingleRight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub(crate) enum WasmTextureSource {
    SampleSkin,
    YourSkin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub(crate) enum WasmWingsAnimations {
    Normal,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub(crate) enum WasmSnoutStatus {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmSnoutSettings {
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) length: u8,
    pub(crate) offset: u8,
    pub(crate) source: WasmTextureSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmWingSettings {
    pub(crate) mode: WasmWingsMode,
    pub(crate) animations: WasmWingsAnimations,
    pub(crate) wings: Option<ByteBuf>,
    pub(crate) source: WasmTextureSource,
}

impl From<WasmWingSettings> for WingData {
    fn from(settings: WasmWingSettings) -> Self {
        Self {
            mode: settings
                .wings
                .map(|_| settings.mode.into())
                .unwrap_or(WingMode::None),
            animated: settings.animations == WasmWingsAnimations::Normal,
        }
    }
}

impl From<WingData> for WasmWingSettings {
    fn from(data: WingData) -> Self {
        Self {
            mode: data.mode.into(),
            animations: if data.animated {
                WasmWingsAnimations::Normal
            } else {
                WasmWingsAnimations::None
            },
            wings: None,
            source: WasmTextureSource::SampleSkin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmTailSettings {
    pub(crate) mode: WasmTailMode,
    pub(crate) segments: u8,
    pub(crate) bends: [f32; 4],
}

impl From<WasmTailSettings> for TailData {
    fn from(settings: WasmTailSettings) -> Self {
        Self {
            mode: settings.mode.into(),
            segments: settings.segments,
            bends: settings.bends,
        }
    }
}

impl From<TailData> for WasmTailSettings {
    fn from(data: TailData) -> Self {
        Self {
            mode: data.mode.into(),
            segments: data.segments,
            bends: data.bends,
        }
    }
}

impl From<WasmSnoutSettings> for SnoutData {
    fn from(settings: WasmSnoutSettings) -> Self {
        Self {
            offset: settings.offset,
            width: settings.width,
            height: settings.height,
            depth: settings.length,
        }
    }
}

impl From<SnoutData> for WasmSnoutSettings {
    fn from(data: SnoutData) -> Self {
        Self {
            offset: data.offset,
            width: data.width,
            height: data.height,
            length: data.depth,
            source: WasmTextureSource::SampleSkin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmEarsSettings {
    pub(crate) mode: WasmEarsMode,
    pub(crate) anchor: WasmEarsAnchor,
    pub(crate) source: WasmTextureSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmAlfalfaData {
    pub(crate) version: u8,
    pub(crate) data: HashMap<String, ByteBuf>,
}

impl From<WasmAlfalfaData> for AlfalfaData {
    fn from(value: WasmAlfalfaData) -> Self {
        Self::new_raw(
            value.version,
            value
                .data
                .into_iter()
                .map(|(k, v)| (k, v.into_vec()))
                .collect(),
        )
    }
}

impl From<AlfalfaData> for WasmAlfalfaData {
    fn from(value: AlfalfaData) -> Self {
        let (version, data) = value.into_raw();

        Self {
            version,
            data: data
                .into_iter()
                .map(|(k, v)| (k, ByteBuf::from(v)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmEarsEmissiveData {
    pub(crate) enabled: bool,
    pub(crate) palette: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WasmEarsFeatures {
    pub(crate) ears: WasmEarsSettings,
    pub(crate) protrusions: Vec<WasmProtrusion>,
    pub(crate) protrusions_source: WasmTextureSource,
    pub(crate) tail: WasmTailSettings,
    pub(crate) snout: Option<WasmSnoutSettings>,
    pub(crate) wings: WasmWingSettings,
    pub(crate) cape: Option<ByteBuf>,
    pub(crate) chest_size: f32,
    pub(crate) alfalfa: Option<WasmAlfalfaData>,
    pub(crate) emissives: WasmEarsEmissiveData,
    pub(crate) data_version: u8,
}

fn rbg_to_hex(image::Rgb([r, g, b]): image::Rgb<u8>) -> u32 {
    u32::from_be_bytes([0xFF, r, g, b])
}

fn hex_to_rgb(hex: u32) -> image::Rgb<u8> {
    image::Rgb::from([((hex >> 16) & 0xFF) as u8, ((hex >> 8) & 0xFF) as u8, (hex & 0xFF) as u8])
}

impl WasmEarsFeatures {
    pub(crate) fn with_emissive(mut self, palette: Option<EarsEmissivePalette>) -> Self {
        self.emissives.palette.clear();
    
        if let Some(emissive_pixels) = palette.as_ref().map(|p| p.0.clone()) {
            self.emissives.palette.extend(emissive_pixels.into_iter().map(rbg_to_hex));
        }
        
        self
    }
    
    pub(crate) fn with_alfalfa(self, alfalfa: Option<AlfalfaData>) -> Self {
        let wings = alfalfa
            .as_ref()
            .and_then(|a| a.get_data(AlfalfaDataKey::Wings).map(|w| ByteBuf::from(w)));

        Self {
            wings: WasmWingSettings {
                source: wings
                    .as_ref()
                    .map(|_| WasmTextureSource::YourSkin)
                    .unwrap_or(WasmTextureSource::SampleSkin),
                wings,
                ..self.wings
            },
            cape: alfalfa
                .as_ref()
                .and_then(|a| a.get_data(AlfalfaDataKey::Cape).map(|w| ByteBuf::from(w))),
            alfalfa: alfalfa.map(|a| a.into()),
            ..self
        }
    }
}

impl From<WasmEarsFeatures> for ears_rs::features::EarsFeatures {
    fn from(features: WasmEarsFeatures) -> Self {
        Self {
            ear_mode: features.ears.mode.into(),
            ear_anchor: features.ears.anchor.into(),
            tail: Some(features.tail.into()).filter(|_| features.tail.mode != WasmTailMode::None),
            snout: features.snout.map(|s| s.into()),
            wing: Some(features.wings.into()).filter(|w: &WingData| w.mode != WingMode::None),
            claws: features.protrusions.contains(&WasmProtrusion::Claws),
            horn: features.protrusions.contains(&WasmProtrusion::Horns),
            chest_size: features.chest_size,
            cape_enabled: features.cape.is_some(),
            emissive: features.emissives.enabled,
            data_version: features.data_version,
        }
    }
}

impl From<WasmEarsFeatures> for AlfalfaData {
    fn from(value: WasmEarsFeatures) -> Self {
        let mut alfalfa = value
            .alfalfa
            .map(|a| a.into())
            .unwrap_or_else(AlfalfaData::new);

        if let Some(cape) = value.cape {
            alfalfa.set_data(AlfalfaDataKey::Cape, cape.to_vec());
        }

        if let Some(wings) = value.wings.wings {
            alfalfa.set_data(AlfalfaDataKey::Wings, wings.to_vec());
        }

        alfalfa
    }
}

impl From<&WasmEarsFeatures> for EarsEmissivePalette {
    fn from(value: &WasmEarsFeatures) -> Self {
        let palette = value
            .emissives
            .palette
            .iter()
            .map(|&hex| hex_to_rgb(hex))
            .collect();

        Self(palette)
    }
}


impl From<EarsFeatures> for WasmEarsFeatures {
    fn from(features: EarsFeatures) -> Self {
        Self {
            ears: WasmEarsSettings {
                mode: features.ear_mode.into(),
                anchor: features.ear_anchor.into(),
                source: WasmTextureSource::SampleSkin,
            },
            protrusions_source: WasmTextureSource::SampleSkin,
            protrusions: {
                let mut protrusions = Vec::new();
                if features.claws {
                    protrusions.push(WasmProtrusion::Claws);
                }
                if features.horn {
                    protrusions.push(WasmProtrusion::Horns);
                }
                protrusions
            },
            tail: features.tail.map(|t| t.into()).unwrap_or(WasmTailSettings {
                mode: WasmTailMode::None,
                segments: 0,
                bends: [0.0; 4],
            }),
            snout: features.snout.map(|s| s.into()),
            wings: features.wing.map(|w| w.into()).unwrap_or(WasmWingSettings {
                mode: WasmWingsMode::None,
                animations: WasmWingsAnimations::None,
                wings: None,
                source: WasmTextureSource::SampleSkin,
            }),
            cape: None,
            chest_size: features.chest_size,
            alfalfa: None,
            emissives: WasmEarsEmissiveData {
                enabled: features.emissive,
                palette: vec![],
            },
            data_version: features.data_version,
        }
    }
}
