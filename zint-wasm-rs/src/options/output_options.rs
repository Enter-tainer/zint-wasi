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
