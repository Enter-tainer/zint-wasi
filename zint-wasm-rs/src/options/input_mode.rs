use serde::Deserialize;

use crate::error::{Error, ValidationFailiure};

bitflags::bitflags! {
    /// Input modes and options
    #[derive(Debug, Clone, Copy)]
    pub struct InputMode: u32 {
        /// Binary
        const DATA = zint_wasm_sys::DATA_MODE;
        /// UTF-8
        const UNICODE = zint_wasm_sys::UNICODE_MODE;
        /// GS1
        const GS1 = zint_wasm_sys::GS1_MODE;

        /// Process escape sequences
        const ESCAPE = zint_wasm_sys::ESCAPE_MODE;
        /// Process parentheses as GS1 AI delimiters (instead of square brackets)
        const GS1_PARENTHESES = zint_wasm_sys::GS1PARENS_MODE;
        /// Do not check validity of GS1 data
        const GS1_NO_CHECK = zint_wasm_sys::GS1NOCHECK_MODE;
        /// Interpret `height` as per-row rather than as overall height
        const HEIGHT_PER_ROW = zint_wasm_sys::HEIGHTPERROW_MODE;
        /// Use faster, less optimal encoding or other shortcuts if available
        const FAST = zint_wasm_sys::FAST_MODE;
        /// Process special symbology-specific escape sequences
        const EXTRA_ESCAPE = zint_wasm_sys::EXTRA_ESCAPE_MODE;
    }
}

impl InputMode {
    pub fn as_i32(&self) -> i32 {
        self.bits() as i32
    }

    pub fn validate(&self) -> Option<ValidationFailiure> {
        // DATA is 0 so it can't be checked as UNICODE overwrites it
        if self.contains(Self::UNICODE) && self.contains(Self::GS1) {
            return Some(ValidationFailiure::MultipleFormats);
        }

        None
    }
}

impl<'de> Deserialize<'de> for InputMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        macro_rules! decl_names {
            ($($opt: ident: [$($value: literal),+],)+) => {
                fn opt_for_name(name: &str) -> Option<InputMode> {
                    let lower = name.to_lowercase().replace('_', "-");
                    let clear = lower.strip_suffix("-mode").unwrap_or(&lower);
                    $(
                    if [$($value),+].contains(&clear) {
                        return Some(InputMode::$opt)
                    }
                    )+
                    None
                }
            };
        }

        decl_names![
            DATA: ["data"],
            UNICODE: ["unicode"],
            GS1: ["gs1"],

            ESCAPE: ["escape"],
            GS1_PARENTHESES: ["gs1-parentheses", "gs1paren"],
            GS1_NO_CHECK: ["gs1-no-check", "gs1nocheck"],
            HEIGHT_PER_ROW: ["height-per-row", "heightperrow"],
            FAST: ["fast"],
            EXTRA_ESCAPE: ["extra-escape"],
        ];

        struct InputModeVisitor;
        impl<'de> de::Visitor<'de> for InputModeVisitor {
            type Value = InputMode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("InputMode")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "data" => Ok(InputMode::DATA),
                    "unicode" => Ok(InputMode::UNICODE),
                    "gs1" => Ok(InputMode::GS1),
                    _ => Err(E::custom(Error::InvalidInputMode(
                        ValidationFailiure::UnknownFormat,
                    ))),
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v > u32::MAX as u64 {
                    return Err(E::custom(Error::InvalidInputMode(
                        ValidationFailiure::TooBig,
                    )));
                }
                Ok(InputMode::from_bits_retain(v as u32))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v > u32::MAX as i64 {
                    return Err(E::custom(Error::InvalidInputMode(
                        ValidationFailiure::TooBig,
                    )));
                } else if v.is_negative() {
                    return Err(E::custom(Error::InvalidInputMode(
                        ValidationFailiure::Negative,
                    )));
                }
                Ok(InputMode::from_bits_retain(v as u32))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut result = InputMode::empty();
                // Owned, because a self describing format such as CBOR decodes
                // into a buffer of its own and has nothing to lend out.
                while let Some(el) = seq.next_element::<String>()? {
                    result = match opt_for_name(&el) {
                        Some(it) => result.union(it),
                        None => return Err(de::Error::custom(Error::UnknownInputOption(el))),
                    }
                }

                Ok(result)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut result = InputMode::empty();

                while let Some(key) = map.next_key::<String>()? {
                    result = match opt_for_name(&key) {
                        Some(it) => {
                            if map.next_value()? {
                                result.union(it)
                            } else {
                                result
                            }
                        }
                        None if key == "format" => {
                            let value = map.next_value::<String>().map_err(|_| {
                                de::Error::custom(Error::InvalidInputMode(
                                    ValidationFailiure::UnknownFormat,
                                ))
                            })?;
                            match value.as_str() {
                                "data" => InputMode::DATA,
                                "unicode" => InputMode::UNICODE,
                                "gs1" => InputMode::GS1,
                                _ => {
                                    return Err(de::Error::custom(Error::InvalidInputMode(
                                        ValidationFailiure::UnknownFormat,
                                    )))
                                }
                            }
                        }
                        None => {
                            return Err(de::Error::custom(Error::UnknownOutputOption(
                                key.to_string(),
                            )))
                        }
                    }
                }
                Ok(result)
            }
        }

        let result = deserializer.deserialize_any(InputModeVisitor)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::InputMode;
    use crate::{error::ValidationFailiure, test_support::from_cbor};
    use ciborium::cbor;

    /// The manual promises that an input format can be given as an integer, a
    /// string, a one element array or a `format` entry in a dictionary, and
    /// that all four mean the same thing.
    #[test]
    fn the_four_documented_forms_of_an_input_format_agree() {
        for (name, bits) in [("data", 0), ("unicode", 1), ("gs1", 2)] {
            let as_int: InputMode = from_cbor(cbor!(bits).unwrap()).expect("integer form");
            let as_string: InputMode = from_cbor(cbor!(name).unwrap()).expect("string form");
            let as_array: InputMode = from_cbor(cbor!([name]).unwrap()).expect("array form");
            let as_dictionary: InputMode =
                from_cbor(cbor!({"format" => name}).unwrap()).expect("dictionary form");

            assert_eq!(as_int.bits(), bits as u32, "integer form of {name}");
            assert_eq!(as_string.bits(), bits as u32, "string form of {name}");
            assert_eq!(as_array.bits(), bits as u32, "array form of {name}");
            assert_eq!(
                as_dictionary.bits(),
                bits as u32,
                "dictionary form of {name}"
            );
        }
    }

    /// A GS1 barcode whose Application Identifiers are written in parentheses,
    /// with escape sequences enabled: flags that have to end up in one integer.
    #[test]
    fn an_array_unions_every_flag_it_lists() {
        let mode: InputMode = from_cbor(cbor!(["gs1", "gs1-parentheses", "escape"]).unwrap())
            .expect("array of flag names");

        assert_eq!(
            mode.bits(),
            (InputMode::GS1 | InputMode::GS1_PARENTHESES | InputMode::ESCAPE).bits()
        );
    }

    /// A GS1 barcode whose Application Identifiers are written in parentheses,
    /// with escape sequences enabled: flags that have to end up in one integer.
    #[test]
    fn a_dictionary_unions_the_flags_marked_true() {
        let mode: InputMode = from_cbor(
            cbor!({
                "format" => "gs1",
                "gs1-parentheses" => true,
                "escape" => true,
                "fast" => false,
            })
            .unwrap(),
        )
        .expect("dictionary of flags");

        assert_eq!(
            mode.bits(),
            (InputMode::GS1 | InputMode::GS1_PARENTHESES | InputMode::ESCAPE).bits()
        );
    }

    #[test]
    fn flag_names_ignore_case_and_separator_style() {
        for spelling in ["height-per-row", "HEIGHT_PER_ROW", "Height-Per-Row"] {
            let mode: InputMode = from_cbor(cbor!({spelling => true}).unwrap())
                .unwrap_or_else(|error| panic!("{spelling:?} should name a flag: {error}"));
            assert_eq!(mode.bits(), InputMode::HEIGHT_PER_ROW.bits());
        }
    }

    /// Zint's own constant names carry a `_MODE` suffix, which is accepted as
    /// well so that values copied out of its documentation work.
    #[test]
    fn the_mode_suffix_of_zints_constant_names_is_accepted() {
        let mode: InputMode =
            from_cbor(cbor!({"ESCAPE_MODE" => true, "FAST_MODE" => true}).unwrap())
                .expect("constant names");

        assert_eq!(mode.bits(), (InputMode::ESCAPE | InputMode::FAST).bits());
    }

    #[test]
    fn an_unknown_flag_name_is_rejected() {
        let from_dictionary =
            from_cbor::<InputMode>(cbor!({"gs2" => true}).unwrap()).expect_err("gs2 is not a flag");
        assert!(
            from_dictionary.contains("gs2"),
            "unexpected error: {from_dictionary}"
        );

        let from_array =
            from_cbor::<InputMode>(cbor!(["gs1", "gs2"]).unwrap()).expect_err("gs2 is not a flag");
        assert!(from_array.contains("gs2"), "unexpected error: {from_array}");
    }

    #[test]
    fn an_unknown_input_format_is_rejected() {
        let from_string =
            from_cbor::<InputMode>(cbor!("utf-8").unwrap()).expect_err("utf-8 is not a format");
        assert!(
            from_string.contains("unknown input format"),
            "unexpected error: {from_string}"
        );

        let from_dictionary = from_cbor::<InputMode>(cbor!({"format" => "utf-8"}).unwrap())
            .expect_err("utf-8 is not a format");
        assert!(
            from_dictionary.contains("unknown input format"),
            "unexpected error: {from_dictionary}"
        );
    }

    /// The integer form is the escape hatch for flags this wrapper does not
    /// know about, so bits it cannot name are passed through rather than
    /// dropped.
    #[test]
    fn an_integer_is_taken_as_a_bit_field_verbatim() {
        let known: InputMode = from_cbor(cbor!(10).unwrap()).expect("GS1 plus escape");
        assert_eq!(known.bits(), (InputMode::GS1 | InputMode::ESCAPE).bits());

        let unknown: InputMode = from_cbor(cbor!(0x8000).unwrap()).expect("unknown bit");
        assert_eq!(unknown.bits(), 0x8000);
    }

    #[test]
    fn an_integer_outside_the_bit_field_is_rejected() {
        let negative =
            from_cbor::<InputMode>(cbor!(-1).unwrap()).expect_err("a bit field is not negative");
        assert!(
            negative.contains("value is negative"),
            "unexpected error: {negative}"
        );

        let too_big = from_cbor::<InputMode>(cbor!(u32::MAX as u64 + 1).unwrap())
            .expect_err("the bit field is 32 bits wide");
        assert!(
            too_big.contains("value is too large"),
            "unexpected error: {too_big}"
        );

        let widest: InputMode = from_cbor(cbor!(u32::MAX as u64).unwrap())
            .expect("the widest bit field there is still fits");
        assert_eq!(widest.bits(), u32::MAX);
    }

    /// A format that reports a positive number as a signed integer, which CBOR
    /// does not, has to be held to the same bound.
    #[test]
    fn a_signed_integer_is_held_to_the_same_bound() {
        use serde::{de::value::I64Deserializer, Deserialize};

        let widest = InputMode::deserialize(I64Deserializer::<serde::de::value::Error>::new(
            u32::MAX as i64,
        ))
        .expect("the widest bit field there is still fits");
        assert_eq!(widest.bits(), u32::MAX);

        let too_big = InputMode::deserialize(I64Deserializer::<serde::de::value::Error>::new(
            u32::MAX as i64 + 1,
        ))
        .expect_err("the bit field is 32 bits wide");
        assert!(
            too_big.to_string().contains("value is too large"),
            "unexpected error: {too_big}"
        );
    }

    /// What a document gets back when the option is not one of the shapes the
    /// manual describes.
    #[test]
    fn a_value_that_is_not_an_input_mode_at_all_is_rejected() {
        let error =
            from_cbor::<InputMode>(cbor!(true).unwrap()).expect_err("a flag is not an input mode");
        assert!(
            error.contains("expected InputMode"),
            "the error should say what was expected: {error}"
        );
    }

    /// These numbers are zint's public ABI: the bits are passed straight into
    /// `symbol->input_mode`, so they have to keep matching `zint.h`.
    #[test]
    fn as_i32_hands_zint_the_raw_bit_field() {
        assert_eq!(InputMode::DATA.as_i32(), 0);
        assert_eq!(InputMode::UNICODE.as_i32(), 1);
        assert_eq!(InputMode::GS1.as_i32(), 2);
        assert_eq!((InputMode::GS1 | InputMode::ESCAPE).as_i32(), 10);
        assert_eq!(InputMode::EXTRA_ESCAPE.as_i32(), 0x100);
        assert_eq!(InputMode::from_bits_retain(u32::MAX).as_i32(), -1);
    }

    /// The three input formats are mutually exclusive, but `DATA` is zero and
    /// so cannot be told apart from "not set".
    #[test]
    fn validate_only_rejects_two_formats_it_can_see() {
        assert!(matches!(
            (InputMode::UNICODE | InputMode::GS1).validate(),
            Some(ValidationFailiure::MultipleFormats)
        ));

        assert!(InputMode::UNICODE.validate().is_none());
        assert!(InputMode::GS1.validate().is_none());
        assert!((InputMode::DATA | InputMode::GS1).validate().is_none());
        assert!((InputMode::GS1 | InputMode::ESCAPE).validate().is_none());
    }
}
