use serde::Deserialize;

use crate::error::{Error, ValidationFailiure};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct OutputOptions: u32 {
        /// Boundary bar above the symbol only (not below), does not affect stacking
        const BARCODE_BIND_TOP = zint_wasm_sys::BARCODE_BIND_TOP;
        /// Boundary bars above & below the symbol and between stacked symbols
        const BARCODE_BIND = zint_wasm_sys::BARCODE_BIND;
        /// Box around symbol
        const BARCODE_BOX = zint_wasm_sys::BARCODE_BOX;
        /// Output to stdout
        const BARCODE_STDOUT = zint_wasm_sys::BARCODE_STDOUT;
        /// Reader Initialisation (Programming)
        const READER_INIT = zint_wasm_sys::READER_INIT;
        /// Use smaller font
        const SMALL_TEXT = zint_wasm_sys::SMALL_TEXT;
        /// Use bold font
        const BOLD_TEXT = zint_wasm_sys::BOLD_TEXT;
        /// CMYK colour space (Encapsulated PostScript and TIF)
        const CMYK_COLOR = zint_wasm_sys::CMYK_COLOUR;
        /// Plot a matrix symbol using dots rather than squares
        const BARCODE_DOTTY_MODE = zint_wasm_sys::BARCODE_DOTTY_MODE;
        /// Use GS instead of FNC1 as GS1 separator (Data Matrix)
        const GS1_GS_SEPARATOR = zint_wasm_sys::GS1_GS_SEPARATOR;
        /// Return ASCII values in bitmap buffer (OUT_BUFFER only)
        const OUT_BUFFER_INTERMEDIATE = zint_wasm_sys::OUT_BUFFER_INTERMEDIATE;
        /// Add compliant quiet zones (additional to any specified whitespace)
        const BARCODE_QUIET_ZONES = zint_wasm_sys::BARCODE_QUIET_ZONES;
        /// Disable quiet zones, notably those with defaults as listed above
        const BARCODE_NO_QUIET_ZONES = zint_wasm_sys::BARCODE_NO_QUIET_ZONES;
        /// Warn if height not compliant, or use standard height (if any) as default
        const COMPLIANT_HEIGHT = zint_wasm_sys::COMPLIANT_HEIGHT;
        /// Add quiet zone indicators ("<"/">") to HRT whitespace (EAN/UPC)
        const EAN_UPC_GUARD_WHITESPACE = zint_wasm_sys::EANUPC_GUARD_WHITESPACE;
        /// Embed font in vector output - currently only for SVG output
        const EMBED_VECTOR_FONT = zint_wasm_sys::EMBED_VECTOR_FONT;
        /// Write output to in-memory buffer `memfile` instead of to `outfile`
        const BARCODE_MEMORY_FILE = zint_wasm_sys::BARCODE_MEMORY_FILE;
    }
}

impl OutputOptions {
    pub fn as_i32(&self) -> i32 {
        self.bits() as i32
    }
}

impl<'de> Deserialize<'de> for OutputOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        macro_rules! decl_names {
            ($($opt: ident: [$($value: literal),+],)+) => {
                fn opt_for_name(name: &str) -> Option<OutputOptions> {
                    let lower = name.to_lowercase().replace('_', "-");
                    $(
                    if [$($value),+].contains(&lower.as_str()) {
                        return Some(OutputOptions::$opt)
                    }
                    )+
                    None
                }
            };
        }

        decl_names![
            BARCODE_BIND_TOP: ["barcode-bind-top"],
            BARCODE_BIND: ["barcode-bind"],
            BARCODE_BOX: ["barcode-box"],
            BARCODE_STDOUT: ["barcode-stdout"],
            READER_INIT: ["reader-init"],
            SMALL_TEXT: ["small-text"],
            BOLD_TEXT: ["bold-text"],
            CMYK_COLOR: ["cmyk-color", "cmyk-colour"],
            BARCODE_DOTTY_MODE: ["barcode-dotty-mode"],
            GS1_GS_SEPARATOR: ["gs1-gs-separator"],
            OUT_BUFFER_INTERMEDIATE: ["out-buffer-intermediate"],
            BARCODE_QUIET_ZONES: ["barcode-quiet-zones"],
            BARCODE_NO_QUIET_ZONES: ["barcode-no-quiet-zones"],
            COMPLIANT_HEIGHT: ["compliant-height"],
            EAN_UPC_GUARD_WHITESPACE: ["ean-upc-guard-whitespace", "eanupc-guard-whitespace"],
            EMBED_VECTOR_FONT: ["embed-vector-font"],
        ];

        struct OutputOptionsVisitor;
        impl<'de> de::Visitor<'de> for OutputOptionsVisitor {
            type Value = OutputOptions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("OutputOptions")
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
                Ok(OutputOptions::from_bits_retain(v as u32))
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
                Ok(OutputOptions::from_bits_retain(v as u32))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut result = OutputOptions::empty();
                while let Some(el) = seq.next_element::<&str>()? {
                    result = match opt_for_name(el) {
                        Some(it) => result.union(it),
                        None => {
                            return Err(de::Error::custom(Error::UnknownOutputOption(
                                el.to_string(),
                            )))
                        }
                    }
                }

                Ok(result)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut result = OutputOptions::empty();

                while let Some(key) = map.next_key::<String>()? {
                    result = match opt_for_name(&key) {
                        Some(it) => {
                            if map.next_value()? {
                                result.union(it)
                            } else {
                                result
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

        deserializer.deserialize_any(OutputOptionsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::OutputOptions;
    use crate::test_support::from_cbor;
    use ciborium::cbor;

    /// A boxed symbol with a bold human readable text: flags that have to end
    /// up in the same integer.
    #[test]
    fn a_dictionary_unions_the_options_marked_true() {
        let options: OutputOptions = from_cbor(
            cbor!({
                "barcode-bind" => true,
                "barcode-box" => true,
                "bold-text" => true,
                "small-text" => false,
            })
            .unwrap(),
        )
        .expect("dictionary of options");

        assert_eq!(
            options.bits(),
            (OutputOptions::BARCODE_BIND | OutputOptions::BARCODE_BOX | OutputOptions::BOLD_TEXT)
                .bits()
        );
    }

    #[test]
    fn option_names_ignore_case_and_separator_style() {
        for spelling in ["compliant-height", "COMPLIANT_HEIGHT", "Compliant-Height"] {
            let options: OutputOptions = from_cbor(cbor!({spelling => true}).unwrap())
                .unwrap_or_else(|error| panic!("{spelling:?} should name an option: {error}"));
            assert_eq!(options.bits(), OutputOptions::COMPLIANT_HEIGHT.bits());
        }
    }

    /// Zint spells colour the British way; both spellings are accepted so that
    /// the name does not have to be looked up.
    #[test]
    fn both_spellings_of_colour_are_accepted() {
        for spelling in ["cmyk-color", "cmyk-colour"] {
            let options: OutputOptions = from_cbor(cbor!({spelling => true}).unwrap())
                .unwrap_or_else(|error| panic!("{spelling:?} should name an option: {error}"));
            assert_eq!(options.bits(), OutputOptions::CMYK_COLOR.bits());
        }
    }

    #[test]
    fn an_unknown_option_name_is_rejected() {
        let error = from_cbor::<OutputOptions>(cbor!({"barcode-round" => true}).unwrap())
            .expect_err("barcode-round is not an option");
        assert!(error.contains("barcode-round"), "unexpected error: {error}");
    }

    /// The plugin always renders to an in-memory file, so that flag is the
    /// library's to set and is deliberately not one a caller can name.
    #[test]
    fn the_memory_file_flag_cannot_be_requested_by_name() {
        assert!(
            from_cbor::<OutputOptions>(cbor!({"barcode-memory-file" => true}).unwrap()).is_err(),
            "the memory file flag is set by the library, not by the caller"
        );
    }

    /// The integer form is the escape hatch for flags this wrapper does not
    /// know about, so bits it cannot name are passed through rather than
    /// dropped.
    #[test]
    fn an_integer_is_taken_as_a_bit_field_verbatim() {
        let known: OutputOptions = from_cbor(cbor!(6).unwrap()).expect("bind plus box");
        assert_eq!(
            known.bits(),
            (OutputOptions::BARCODE_BIND | OutputOptions::BARCODE_BOX).bits()
        );

        let unknown: OutputOptions = from_cbor(cbor!(0x100000).unwrap()).expect("unknown bit");
        assert_eq!(unknown.bits(), 0x100000);
    }

    #[test]
    fn an_integer_outside_the_bit_field_is_rejected() {
        let negative = from_cbor::<OutputOptions>(cbor!(-1).unwrap())
            .expect_err("a bit field is not negative");
        assert!(
            negative.contains("value is negative"),
            "unexpected error: {negative}"
        );

        let too_big = from_cbor::<OutputOptions>(cbor!(u32::MAX as u64 + 1).unwrap())
            .expect_err("the bit field is 32 bits wide");
        assert!(
            too_big.contains("value is too large"),
            "unexpected error: {too_big}"
        );

        let widest: OutputOptions = from_cbor(cbor!(u32::MAX as u64).unwrap())
            .expect("the widest bit field there is still fits");
        assert_eq!(widest.bits(), u32::MAX);
    }

    /// A format that reports a positive number as a signed integer, which CBOR
    /// does not, has to be held to the same bound.
    #[test]
    fn a_signed_integer_is_held_to_the_same_bound() {
        use serde::{de::value::I64Deserializer, Deserialize};

        let widest = OutputOptions::deserialize(I64Deserializer::<serde::de::value::Error>::new(
            u32::MAX as i64,
        ))
        .expect("the widest bit field there is still fits");
        assert_eq!(widest.bits(), u32::MAX);

        let too_big = OutputOptions::deserialize(I64Deserializer::<serde::de::value::Error>::new(
            u32::MAX as i64 + 1,
        ))
        .expect_err("the bit field is 32 bits wide");
        assert!(
            too_big.to_string().contains("value is too large"),
            "unexpected error: {too_big}"
        );
    }

    /// A bare string is not one of the documented forms; option names have to
    /// arrive in a dictionary.
    #[test]
    fn a_bare_string_is_rejected() {
        let error = from_cbor::<OutputOptions>(cbor!("barcode-box").unwrap())
            .expect_err("one option is still a dictionary or an array");
        assert!(
            error.contains("expected OutputOptions"),
            "the error should say what was expected: {error}"
        );
    }

    /// These numbers are zint's public ABI: the flags are passed straight into
    /// `symbol->output_options`, so they have to keep matching `zint.h`.
    #[test]
    fn the_flags_match_the_values_zint_defines() {
        assert_eq!(OutputOptions::BARCODE_BIND_TOP.bits(), 0x00001);
        assert_eq!(OutputOptions::BARCODE_BIND.bits(), 0x00002);
        assert_eq!(OutputOptions::BARCODE_BOX.bits(), 0x00004);
        assert_eq!(OutputOptions::BARCODE_DOTTY_MODE.bits(), 0x00100);
        assert_eq!(OutputOptions::COMPLIANT_HEIGHT.bits(), 0x02000);
        assert_eq!(OutputOptions::EMBED_VECTOR_FONT.bits(), 0x08000);
        assert_eq!(OutputOptions::BARCODE_MEMORY_FILE.bits(), 0x10000);
    }

    #[test]
    fn as_i32_hands_zint_the_raw_bit_field() {
        assert_eq!(OutputOptions::empty().as_i32(), 0);
        assert_eq!(
            (OutputOptions::BARCODE_BIND | OutputOptions::BARCODE_BOX).as_i32(),
            6
        );
        assert_eq!(OutputOptions::from_bits_retain(u32::MAX).as_i32(), -1);
    }
}
