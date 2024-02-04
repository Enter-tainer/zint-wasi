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
                while let Some(el) = seq.next_element::<&str>()? {
                    result = match opt_for_name(el) {
                        Some(it) => result.union(it),
                        None => {
                            return Err(de::Error::custom(Error::UnknownInputOption(
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
