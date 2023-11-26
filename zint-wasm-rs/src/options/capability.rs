use serde::{Deserialize, Serialize};
use zint_wasm_sys::*;

/// Capability flags (ZBarcode_Cap() `cap_flag`)
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
#[serde(tag = "type")]
pub enum CapabilityFlags {
    /// Prints Human Readable Text?
    HRT,
    /// Is stackable?
    Stackable,
    /// Is EAN/UPC?
    EanUpc,
    /// Legacy
    Extendable,
    /// Can have composite data?
    Composite,
    /// Supports Extended Channel Interpretations?
    Eci,
    /// Supports GS1 data?
    Gs1,
    /// Can be output as dots?
    Dotty,
    /// Has default quiet zones?
    QuietZones,
    /// Has fixed width-to-height (aspect) ratio?
    FixedRatio,
    /// Supports Reader Initialisation?
    ReaderInit,
    /// Supports full-multibyte option?
    FullMultibyte,
    /// Is mask selectable?
    Mask,
    /// Supports Structured Append?
    StructApp,
    /// Has compliant height?
    CompliantHeight,
}

impl From<CapabilityFlags> for i32 {
    fn from(val: CapabilityFlags) -> Self {
        match val {
            CapabilityFlags::HRT => ZINT_CAP_HRT,
            CapabilityFlags::Stackable => ZINT_CAP_STACKABLE,
            CapabilityFlags::EanUpc => ZINT_CAP_EANUPC,
            CapabilityFlags::Extendable => ZINT_CAP_EXTENDABLE,
            CapabilityFlags::Composite => ZINT_CAP_COMPOSITE,
            CapabilityFlags::Eci => ZINT_CAP_ECI,
            CapabilityFlags::Gs1 => ZINT_CAP_GS1,
            CapabilityFlags::Dotty => ZINT_CAP_DOTTY,
            CapabilityFlags::QuietZones => ZINT_CAP_QUIET_ZONES,
            CapabilityFlags::FixedRatio => ZINT_CAP_FIXED_RATIO,
            CapabilityFlags::ReaderInit => ZINT_CAP_READER_INIT,
            CapabilityFlags::FullMultibyte => ZINT_CAP_FULL_MULTIBYTE,
            CapabilityFlags::Mask => ZINT_CAP_MASK,
            CapabilityFlags::StructApp => ZINT_CAP_STRUCTAPP,
            CapabilityFlags::CompliantHeight => ZINT_CAP_COMPLIANT_HEIGHT,
        }
        .try_into()
        .unwrap()
    }
}
