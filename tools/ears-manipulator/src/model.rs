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

use ears_rs::features::data::{ear::{EarMode, EarAnchor}, tail::{TailMode, TailData}, wing::{WingMode, WingData}, snout::SnoutData};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum WasmProtrusion {
    Claws,
    Horns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum WasmWingsAnimations {
    Normal,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum WasmSnoutStatus {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub(crate) struct WasmSnoutSettings {
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) length: u8,
    pub(crate) offset: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub(crate) struct WasmEarsSettings {
    pub(crate) mode: WasmEarsMode,
    pub(crate) anchor: WasmEarsAnchor,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
            wing: Some(features.wings.into()).filter(|_| features.wings.mode != WasmWingsMode::None),
            claws: features.protrusions.contains(&WasmProtrusion::Claws),
            horn: features.protrusions.contains(&WasmProtrusion::Horns),
            chest_size: 0.0, // TODO
            cape_enabled: false, // TODO
            emissive: false, // TODO
        }
    }
}