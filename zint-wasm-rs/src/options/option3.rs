use std::fmt::Debug;

use serde::Deserialize;
use zint_wasm_sys::*;

use crate::error::Error;

/// The part of `option_3` zint reads as the Data Matrix shape, which is a
/// choice between three values rather than a set of bits.
const DM_SHAPE: u32 = 0x7F;

bitflags::bitflags! {
    /// Data Matrix specific options
    ///
    /// Not every combination of these is meaningful, which is why this is not
    /// an enum: the low seven bits hold the shape as a *value*, and only
    /// `ISO_144` is a bit of its own. `SQUARE` and `DMRE` are alternatives and
    /// differ in a single bit, so combining them or asking `contains` about
    /// them answers the wrong question. Use [`DataMatrixOption::shape`].
    #[derive(Debug, Clone, Copy, Deserialize)]
    #[serde(try_from = "u32")]
    pub struct DataMatrixOption: u32 {
        /// Only consider square versions on automatic symbol size selection
        const SQUARE = DM_SQUARE;
        /// Consider DMRE versions on automatic symbol size selection
        const DMRE = DM_DMRE;
        /// Use ISO instead of "de facto" format for 144x144 (i.e. don't skew ECC)
        const ISO_144 = DM_ISO_144;
    }
}

impl DataMatrixOption {
    /// The shape zint will read out of this value: `DM_SQUARE`, `DM_DMRE`, or
    /// zero for the size it would have chosen anyway.
    ///
    /// `contains` cannot answer this and will say yes when it should not:
    /// `DM_SQUARE` is `0b1100100` and `DM_DMRE` is `0b1100101`, so a DMRE value
    /// contains every bit of a square one.
    pub fn shape(self) -> u32 {
        self.bits() & DM_SHAPE
    }
}

impl TryFrom<u32> for DataMatrixOption {
    type Error = Error;

    /// The value carries two independent fields in one integer: the low seven
    /// bits choose the shape and are either nothing, `DM_SQUARE` or `DM_DMRE`,
    /// and the eighth bit is `DM_ISO_144`. Zint reads them separately, so
    /// "square only, and place the 144x144 ECC the ISO way" is a request it
    /// honours and 228 is how it is spelled.
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let shape = value & DM_SHAPE;
        let unknown = value & !(DM_SHAPE | DM_ISO_144);

        // Zero passes: it sets no shape and no flag, which is the size zint
        // would have chosen anyway, and is what leaving `option_3` out means.
        let recognised = unknown == 0 && (shape == 0 || shape == DM_SQUARE || shape == DM_DMRE);

        if recognised {
            Ok(DataMatrixOption::from_bits_retain(value))
        } else {
            Err(Error::UnknownOption {
                which: "option_3",
                value: Box::new(value),
            })
        }
    }
}

/// QR mask used to minimize unwanted patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QRMask {
    #[doc = include_str!("../../../assets/masks/mask000.svg")]
    ///
    /// Applies a mask where modules alternate between dark and light every
    /// other module in both rows and columns.
    ///
    /// Formula: `(i + j) % 2 = 0`
    Mask0 = 0b000,
    #[doc = include_str!("../../../assets/masks/mask001.svg")]
    ///
    /// Modules alternate every other column.
    ///
    /// Formula: `i % 2 = 0`
    Mask1 = 0b001,
    #[doc = include_str!("../../../assets/masks/mask010.svg")]
    ///
    /// Alternates every other row.
    ///
    /// Formula: `j % 3 = 0`
    Mask2 = 0b010,
    #[doc = include_str!("../../../assets/masks/mask011.svg")]
    ///
    /// Alternates based on a combination of both rows and columns but with a
    /// more complex formula.
    ///
    /// Formula: `(i + j) % 3 = 0`
    Mask3 = 0b011,
    #[doc = include_str!("../../../assets/masks/mask100.svg")]
    ///
    /// Modules change depending on their diagonal position.
    ///
    /// Formula: `(i/2 + j/3) % 2 = 0`
    Mask4 = 0b100,
    #[doc = include_str!("../../../assets/masks/mask101.svg")]
    ///
    /// A specific rule based on the sum of the row and column indices.
    ///
    /// Formula: `(i*j) % 2 + (i*j) % 3 = 0`
    Mask5 = 0b101,
    #[doc = include_str!("../../../assets/masks/mask110.svg")]
    ///
    /// Modules change based on the parity of the row and column.
    ///
    /// Formula: `((i*j) % 3 + (i*j)) % 2 = 0`
    Mask6 = 0b110,
    #[doc = include_str!("../../../assets/masks/mask111.svg")]
    ///
    /// Mask based on position and binary sum of the module's row and column
    /// indices.
    ///
    /// Formula: `((i*j) % 3 + i + j) % 2 = 0`
    Mask7 = 0b111,
}

bitflags::bitflags! {
    /// QR, Han Xin, Grid Matrix specific options
    #[derive(Debug, Clone, Copy, Deserialize)]
    #[serde(transparent)]
    pub struct QRMatrixOption: u32 {
        /// Increase non-ASCII data density
        const FULL_MULITIBYTE = ZINT_FULL_MULTIBYTE;

        /// [Mask 0](QRMask::Mask0) option
        const MASK_0 = (QRMask::Mask0 as u32 + 1) << 8;
        /// [Mask 1](QRMask::Mask1) option
        const MASK_1 = (QRMask::Mask1 as u32 + 1) << 8;
        /// [Mask 2](QRMask::Mask2) option
        const MASK_2 = (QRMask::Mask2 as u32 + 1) << 8;
        /// [Mask 3](QRMask::Mask3) option
        const MASK_3 = (QRMask::Mask3 as u32 + 1) << 8;
        /// [Mask 4](QRMask::Mask4) option
        const MASK_4 = (QRMask::Mask4 as u32 + 1) << 8;
        /// [Mask 5](QRMask::Mask5) option
        const MASK_5 = (QRMask::Mask5 as u32 + 1) << 8;
        /// [Mask 6](QRMask::Mask6) option
        const MASK_6 = (QRMask::Mask6 as u32 + 1) << 8;
        /// [Mask 7](QRMask::Mask7) option
        const MASK_7 = (QRMask::Mask7 as u32 + 1) << 8;
    }
}

impl From<QRMask> for QRMatrixOption {
    fn from(mask: QRMask) -> Self {
        QRMatrixOption::from_bits_retain((mask as u32 + 1) << 8)
    }
}
impl TryFrom<u32> for QRMatrixOption {
    type Error = Error;

    /// The value carries two independent fields in one integer: the low byte
    /// is either nothing or `ZINT_FULL_MULTIBYTE`, and the high byte is a mask
    /// number stored one above itself, so 1 through 8. A number outside that
    /// shape is rejected even when it happens to share bits with one, since
    /// zint would otherwise read it as a request nobody made.
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let compression = value & 0xFF;
        let mask = (value >> 8) & 0xFF;
        let above = value >> 16;

        if above == 0
            && (compression == 0 || compression == ZINT_FULL_MULTIBYTE)
            && mask <= QRMask::Mask7 as u32 + 1
        {
            Ok(QRMatrixOption::from_bits_retain(value))
        } else {
            Err(Error::UnknownOption {
                which: "option_3",
                value: Box::new(value),
            })
        }
    }
}

/// Ultracode specific option
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged, try_from = "u32")]
#[repr(u32)]
pub enum UltracodeOption {
    /// Enable Ultracode compression (experimental)
    Compression = ULTRA_COMPRESSION,
}

impl TryFrom<u32> for UltracodeOption {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            ULTRA_COMPRESSION => Self::Compression,
            other => {
                return Err(Error::UnknownOption {
                    which: "option_3",
                    value: Box::new(other),
                })
            }
        })
    }
}

/// Option3 is an `u32` whose variant is determined by
/// [`Options::symbology`](super::Options::symbology) value.
#[derive(Clone, Copy)]
#[repr(C)]
pub union Option3 {
    pub data_matrix: DataMatrixOption,
    pub qr_matrix: QRMatrixOption,
    pub ultracode: UltracodeOption,
}

impl Option3 {
    pub fn as_i32(&self) -> i32 {
        let result: u32 = unsafe {
            // Safety: All variants are u32
            std::mem::transmute(*self)
        };

        result as i32
    }
    /// # Safety
    /// Option3 can be treated as [`DataMatrixOption`] only when
    /// [`symbology`](super::Options::symbology) stored in parent
    /// [`Options`](super::Options) permits so.
    pub unsafe fn as_data_matrix(&self) -> DataMatrixOption {
        self.data_matrix
    }
    /// # Safety
    /// Option3 can be treated as [`QRMatrixOption`] only when
    /// [`symbology`](super::Options::symbology) stored in parent
    /// [`Options`](super::Options) permits so.
    pub unsafe fn as_qr_matrix(&self) -> QRMatrixOption {
        self.qr_matrix
    }
    /// # Safety
    /// Option3 can be treated as [`UltracodeOption`] only when
    /// [`symbology`](super::Options::symbology) stored in parent
    /// [`Options`](super::Options) permits so.
    pub unsafe fn as_ultracode(&self) -> UltracodeOption {
        self.ultracode
    }
}

impl Debug for Option3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_i32().fmt(f)
    }
}

impl From<DataMatrixOption> for Option3 {
    fn from(value: DataMatrixOption) -> Self {
        unsafe {
            // Safety: DataMatrixOption is a valid Option3 variant
            std::mem::transmute(value)
        }
    }
}
impl From<QRMatrixOption> for Option3 {
    fn from(value: QRMatrixOption) -> Self {
        unsafe {
            // Safety: QRMatrixOption is a valid Option3 variant
            std::mem::transmute(value)
        }
    }
}
impl From<UltracodeOption> for Option3 {
    fn from(value: UltracodeOption) -> Self {
        unsafe {
            // Safety: UltracodeOption is a valid Option3 variant
            std::mem::transmute(value)
        }
    }
}

/// Number of entries in zint's Data Matrix size table, which `option_2`
/// indexes from 1. Only used to recognise a size that arrived at `option_3`.
const DATA_MATRIX_SIZES: u32 = 48;

/// Names the option a rejected value plausibly belongs to, so that the error
/// says what to do rather than only that the value is wrong.
fn misplaced_value_hint(value: u32) -> &'static str {
    if (1..=DATA_MATRIX_SIZES).contains(&value) {
        "; a fixed Data Matrix size belongs in option-2, option-3 only constrains \
         automatic size selection"
    } else {
        ""
    }
}

impl TryFrom<u32> for Option3 {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        // don't care which variant it is, we're just checking that the value
        // can be stored as one
        DataMatrixOption::try_from(value)
            .map(From::<DataMatrixOption>::from)
            .or_else(|_| QRMatrixOption::try_from(value).map(From::<QRMatrixOption>::from))
            .or_else(|_| UltracodeOption::try_from(value).map(From::<UltracodeOption>::from))
            .map_err(|_| Error::UnknownOption3 {
                value,
                hint: misplaced_value_hint(value),
            })
    }
}

impl<'de> Deserialize<'de> for Option3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct Option3Visitor;
        impl<'de> de::Visitor<'de> for Option3Visitor {
            type Value = Option3;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "option_3 value: a name such as \"square\" or \"full-multibyte\", \
                     or the number zint documents for the symbology",
                )
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Option3::try_from(v as u32).map_err(de::Error::custom)
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Option3::try_from(v as u32).map_err(de::Error::custom)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let lower = v.to_lowercase().replace('_', "-");
                Ok(match lower.as_str() {
                    "dm-square" | "square" => Option3::from(DataMatrixOption::SQUARE),
                    "dm-dmre" | "dmre" | "rect" => Option3::from(DataMatrixOption::DMRE),
                    "dm-iso-144" | "iso-144" => Option3::from(DataMatrixOption::ISO_144),
                    "zint-full-multibyte" | "full-multibyte" => {
                        Option3::from(QRMatrixOption::FULL_MULITIBYTE)
                    }
                    "ultra-compression" | "compression" => {
                        Option3::from(UltracodeOption::Compression)
                    }
                    _ => return Err(de::Error::invalid_value(de::Unexpected::Str(v), &self)),
                })
            }
        }

        deserializer.deserialize_any(Option3Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{DataMatrixOption, Option3, QRMask, QRMatrixOption, UltracodeOption};
    use crate::{error::Error, test_support::from_cbor};
    use ciborium::cbor;
    use zint_wasm_sys::{DM_DMRE, DM_SQUARE};

    /// `option_3` is a single integer whose meaning depends on the symbology,
    /// so what these types have to get right is the number that reaches zint.
    #[test]
    fn the_options_match_the_values_zint_defines() {
        assert_eq!(Option3::from(DataMatrixOption::SQUARE).as_i32(), 100);
        assert_eq!(Option3::from(DataMatrixOption::DMRE).as_i32(), 101);
        assert_eq!(Option3::from(DataMatrixOption::ISO_144).as_i32(), 128);
        assert_eq!(Option3::from(QRMatrixOption::FULL_MULITIBYTE).as_i32(), 200);
        assert_eq!(Option3::from(UltracodeOption::Compression).as_i32(), 128);
    }

    /// Zint reads the QR mask from the second byte and treats zero there as
    /// "choose one for me", so every mask is stored one above its number.
    #[test]
    fn qr_masks_are_stored_one_above_their_number() {
        for (mask, expected) in [
            (QRMask::Mask0, 0x100),
            (QRMask::Mask3, 0x400),
            (QRMask::Mask7, 0x800),
        ] {
            assert_eq!(QRMatrixOption::from(mask).bits(), expected, "{mask:?}");
        }

        assert_eq!(
            QRMatrixOption::MASK_0.bits(),
            QRMatrixOption::from(QRMask::Mask0).bits()
        );
        assert_eq!(
            QRMatrixOption::MASK_7.bits(),
            QRMatrixOption::from(QRMask::Mask7).bits()
        );
    }

    /// The Data Matrix shape and the 144x144 ECC placement occupy different
    /// parts of the same integer, which is what lets them be requested
    /// together. Zint reads the shape as `option_3 & 0x7F` and the placement as
    /// `option_3 & DM_ISO_144`, so neither hides the other.
    #[test]
    fn a_shape_and_iso_144_placement_fit_in_the_same_integer() {
        let both = DataMatrixOption::SQUARE | DataMatrixOption::ISO_144;

        assert_eq!(Option3::from(both).as_i32(), 228);
        assert_eq!(both.shape(), DM_SQUARE, "the shape is still square");
        assert!(
            both.contains(DataMatrixOption::ISO_144),
            "the placement is still ISO"
        );

        // `shape()` rather than `contains`, which cannot tell the two shapes
        // apart: DM_DMRE has every bit DM_SQUARE has.
        for (value, shape) in [(228, DM_SQUARE), (229, DM_DMRE)] {
            let parsed = DataMatrixOption::try_from(value)
                .unwrap_or_else(|error| panic!("{value} is a Data Matrix option: {error}"));

            assert_eq!(parsed.bits(), value);
            assert_eq!(parsed.shape(), shape, "{value} names the wrong shape");
            assert!(parsed.contains(DataMatrixOption::ISO_144));
        }
    }

    /// The combinations have to survive the guess `Option3` makes, which tries
    /// each of the three option types in turn and takes the first that accepts.
    #[test]
    fn a_combination_survives_the_option_3_dispatch() {
        for value in [228, 229] {
            let option = Option3::try_from(value)
                .unwrap_or_else(|error| panic!("{value} is an option_3 value: {error}"));

            assert_eq!(option.as_i32(), value as i32);
        }

        // The form a document actually sends.
        let option: Option3 = from_cbor(cbor!(228).unwrap()).expect("228 is an option_3 value");
        assert_eq!(option.as_i32(), 228);
    }

    /// Masking the shape must not turn every integer into a Data Matrix
    /// option: a number whose low seven bits name no shape, or that sets a bit
    /// zint does not read, is still a mistake worth reporting.
    #[test]
    fn a_value_outside_the_two_fields_is_still_rejected() {
        for value in [
            27,  // a fixed size, which belongs in option-2
            155, // 27 with the ISO 144 bit set
            356, // 100 with a bit zint reads for nothing
        ] {
            assert!(
                DataMatrixOption::try_from(value).is_err(),
                "{value} is not a Data Matrix option"
            );
        }
    }

    /// A fixed mask and Kanji compression occupy different bytes of the same
    /// integer, which is what lets them be requested together.
    #[test]
    fn a_mask_and_full_multibyte_fit_in_the_same_integer() {
        let both = QRMatrixOption::FULL_MULITIBYTE | QRMatrixOption::from(QRMask::Mask5);

        assert_eq!(Option3::from(both).as_i32(), 200 | 0x600);
        assert_eq!(
            both.bits() & 0xFF,
            200,
            "the low byte still selects Kanji compression"
        );
        assert_eq!(
            (both.bits() >> 8) - 1,
            QRMask::Mask5 as u32,
            "the high byte still selects the mask"
        );
    }

    #[test]
    fn names_ignore_case_and_separator_style() {
        for (name, expected) in [
            ("dm-square", 100),
            ("DM_SQUARE", 100),
            ("square", 100),
            ("dmre", 101),
            ("rect", 101),
            ("DM_DMRE", 101),
            ("iso-144", 128),
            ("DM_ISO_144", 128),
            ("full-multibyte", 200),
            ("ZINT_FULL_MULTIBYTE", 200),
            ("compression", 128),
            ("ULTRA_COMPRESSION", 128),
        ] {
            let option: Option3 = from_cbor(cbor!(name).unwrap())
                .unwrap_or_else(|error| panic!("{name} should be an option_3 value: {error}"));

            assert_eq!(option.as_i32(), expected, "{name}");
        }
    }

    #[test]
    fn an_unknown_name_is_rejected() {
        let error = from_cbor::<Option3>(cbor!("dm-round").unwrap())
            .expect_err("dm-round is not an option");
        assert!(error.contains("dm-round"), "unexpected error: {error}");
    }

    /// What a document gets back when the option is neither a name nor a
    /// number.
    #[test]
    fn a_value_that_is_not_an_option_at_all_is_rejected() {
        let error = from_cbor::<Option3>(cbor!(true).unwrap())
            .expect_err("a flag is not an option_3 value");
        assert!(
            error.contains("expected option_3 value"),
            "the error should say what was expected: {error}"
        );
    }

    #[test]
    fn integers_that_name_a_known_option_are_accepted() {
        for value in [100, 101, 128, 200, 0x100, 0x800] {
            let option: Option3 = from_cbor(cbor!(value).unwrap())
                .unwrap_or_else(|error| panic!("{value} should be an option_3 value: {error}"));

            assert_eq!(option.as_i32(), value);
        }
    }

    #[test]
    fn an_integer_that_names_nothing_is_rejected() {
        for value in [1, 99, 102, 199, 0x1000] {
            assert!(
                from_cbor::<Option3>(cbor!(value).unwrap()).is_err(),
                "{value} is not an option_3 value"
            );
        }
    }

    /// The three option types are one union, because zint stores them in one
    /// field: 128 means "ISO 144x144" for Data Matrix and "compression" for
    /// Ultracode, and only the symbology says which.
    #[test]
    fn the_option_types_share_a_single_integer() {
        let option = Option3::try_from(128).expect("128 is a known option_3 value");

        assert_eq!(option.as_i32(), 128);
        assert_eq!(
            unsafe {
                // Safety: 128 is a valid value of both types, which is exactly
                // the ambiguity being asserted here.
                option.as_data_matrix()
            }
            .bits(),
            DataMatrixOption::ISO_144.bits()
        );
        assert_eq!(
            unsafe {
                // Safety: see above.
                option.as_ultracode()
            } as u32,
            UltracodeOption::Compression as u32
        );
    }

    #[test]
    fn a_rejected_value_says_which_option_it_belongs_to() {
        for error in [
            DataMatrixOption::try_from(42).expect_err("42 is not a Data Matrix option"),
            QRMatrixOption::try_from(42).expect_err("42 is not a QR option"),
            UltracodeOption::try_from(42).expect_err("42 is not an Ultracode option"),
        ] {
            assert!(
                matches!(
                    error,
                    Error::UnknownOption {
                        which: "option_3",
                        ..
                    }
                ),
                "unexpected error: {error:?}"
            );
        }

        let error = Option3::try_from(42).expect_err("42 is not an option_3 value");
        assert!(
            matches!(error, Error::UnknownOption3 { value: 42, .. }),
            "unexpected error: {error:?}"
        );
    }

    /// A rejected number is most often a Data Matrix size, which selects a
    /// fixed symbol from zint's size table and is an option_2 value. The error
    /// is the only place a document finds that out.
    #[test]
    fn a_data_matrix_size_is_sent_to_option_2() {
        // 12x36, the size reported as not working.
        let error =
            from_cbor::<Option3>(cbor!(28).unwrap()).expect_err("28 is a size, not an option");

        assert!(error.contains("option-2"), "unexpected error: {error}");

        for size in [1, 8, 28, 48] {
            let error = Option3::try_from(size).expect_err("a size is not an option_3 value");
            assert!(
                error.to_string().contains("option-2"),
                "size {size} should point at option-2: {error}"
            );
        }
    }

    /// Above the size table there is nothing to point at, so the error names
    /// the values it does accept and leaves it there.
    #[test]
    fn a_number_that_is_not_a_size_either_lists_the_accepted_values() {
        let error = Option3::try_from(1000).expect_err("1000 is not an option_3 value");
        let message = error.to_string();

        assert!(!message.contains("option-2"), "unexpected error: {message}");
        for expected in ["100", "101", "128", "200", "0x800"] {
            assert!(
                message.contains(expected),
                "the error should name {expected}: {message}"
            );
        }
    }

    /// `option_3` is two fields in one integer, and only some numbers are a
    /// combination of them. A number that merely shares bits with one used to
    /// pass through and reach zint as a request nobody made.
    #[test]
    fn a_number_that_only_shares_bits_with_an_option_is_rejected() {
        for value in [8, 64, 72, 192, 0x1100, 0x900] {
            assert!(
                Option3::try_from(value).is_err(),
                "{value} is not an option_3 value"
            );
        }
    }

    #[test]
    fn a_mask_together_with_full_multibyte_is_accepted() {
        for value in [0x100, 0x800, 200 | 0x100, 200 | 0x600, 200 | 0x800] {
            let option = Option3::try_from(value)
                .unwrap_or_else(|error| panic!("{value:#x} should be an option_3 value: {error}"));

            assert_eq!(option.as_i32(), value as i32);
        }
    }

    /// The union has no discriminant to print, so `Debug` shows the number that
    /// reaches zint instead.
    #[test]
    fn debug_shows_the_number_zint_receives() {
        assert_eq!(
            format!("{:?}", Option3::from(DataMatrixOption::SQUARE)),
            "100"
        );
        assert_eq!(
            format!("{:?}", Option3::from(QRMatrixOption::from(QRMask::Mask7))),
            "2048"
        );
    }
}
