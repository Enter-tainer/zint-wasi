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

#[cfg(test)]
mod tests {
    use super::CapabilityFlags;
    use crate::test_support::from_cbor;
    use ciborium::cbor;

    /// These numbers are zint's public ABI: they are passed straight to
    /// `ZBarcode_Cap`, so they have to keep matching `zint.h`.
    #[test]
    fn every_flag_matches_the_value_zint_defines() {
        for (capability, expected) in [
            (CapabilityFlags::HRT, 0x0001),
            (CapabilityFlags::Stackable, 0x0002),
            (CapabilityFlags::EanUpc, 0x0004),
            (CapabilityFlags::Extendable, 0x0004),
            (CapabilityFlags::Composite, 0x0008),
            (CapabilityFlags::Eci, 0x0010),
            (CapabilityFlags::Gs1, 0x0020),
            (CapabilityFlags::Dotty, 0x0040),
            (CapabilityFlags::QuietZones, 0x0080),
            (CapabilityFlags::FixedRatio, 0x0100),
            (CapabilityFlags::ReaderInit, 0x0200),
            (CapabilityFlags::FullMultibyte, 0x0400),
            (CapabilityFlags::Mask, 0x0800),
            (CapabilityFlags::StructApp, 0x1000),
            (CapabilityFlags::CompliantHeight, 0x2000),
        ] {
            assert_eq!(i32::from(capability), expected, "{capability:?}");
        }
    }

    /// `Extendable` is the name the flag used to have; both spellings have to
    /// keep asking zint the same question.
    #[test]
    fn the_former_name_of_the_ean_upc_flag_asks_the_same_question() {
        assert_eq!(
            i32::from(CapabilityFlags::EanUpc),
            i32::from(CapabilityFlags::Extendable)
        );
    }

    #[test]
    fn a_flag_is_named_by_its_type() {
        let capability: CapabilityFlags = from_cbor(cbor!({"type" => "Mask"}).unwrap())
            .expect("a capability is tagged by its type");

        assert_eq!(i32::from(capability), 0x0800);
    }
}
