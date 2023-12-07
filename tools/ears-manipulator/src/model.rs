/* export interface WasmEarsFeatures {
    ears: EarsSettings;
    protrusions: Set<Protrusion>;
    tail: TailSettings;
    snout?: SnoutSettings;
    wings: WingSettings;
}

export interface WasmEarsSettings {
    mode: EarsMode;
    anchor: EarsAnchor;
}

export interface WasmTailSettings {
    mode: TailMode;
    segments: 1 | 2 | 3 | 4;
    bends: number[];
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
}

export interface WasmWingSettings {
    mode: WingsMode;
    animations: WingsAnimations;
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

use ears_rs::features::{
    data::{
        ear::{EarAnchor, EarMode},
        snout::SnoutData,
        tail::{TailData, TailMode},
        wing::{WingData, WingMode},
    },
    EarsFeatures,
};
use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmWingSettings {
    pub(crate) mode: WasmWingsMode,
    pub(crate) animations: WasmWingsAnimations,
}

impl From<WasmWingSettings> for WingData {
    fn from(settings: WasmWingSettings) -> Self {
        Self {
            mode: settings.mode.into(),
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmEarsSettings {
    pub(crate) mode: WasmEarsMode,
    pub(crate) anchor: WasmEarsAnchor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasmEarsFeatures {
    pub(crate) ears: WasmEarsSettings,
    pub(crate) protrusions: Vec<WasmProtrusion>,
    pub(crate) tail: WasmTailSettings,
    pub(crate) snout: Option<WasmSnoutSettings>,
    pub(crate) wings: WasmWingSettings,
}

impl From<WasmEarsFeatures> for ears_rs::features::EarsFeatures {
    fn from(features: WasmEarsFeatures) -> Self {
        Self {
            ear_mode: features.ears.mode.into(),
            ear_anchor: features.ears.anchor.into(),
            tail: Some(features.tail.into()).filter(|_| features.tail.mode != WasmTailMode::None),
            snout: features.snout.map(|s| s.into()),
            wing: Some(features.wings.into())
                .filter(|_| features.wings.mode != WasmWingsMode::None),
            claws: features.protrusions.contains(&WasmProtrusion::Claws),
            horn: features.protrusions.contains(&WasmProtrusion::Horns),
            chest_size: 0.0,     // TODO
            cape_enabled: false, // TODO
            emissive: false,     // TODO
        }
    }
}

impl From<EarsFeatures> for WasmEarsFeatures {
    fn from(features: EarsFeatures) -> Self {
        Self {
            ears: WasmEarsSettings {
                mode: features.ear_mode.into(),
                anchor: features.ear_anchor.into(),
            },
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
            }),
            // TODO: chest_size, cape_enabled, emissive
        }
    }
}
