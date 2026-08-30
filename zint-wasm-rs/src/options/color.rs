use std::str::FromStr;

use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: u8::MAX,
    };
    pub const TRANSPARENT: Color = Color {
        r: u8::MAX,
        g: u8::MAX,
        b: u8::MAX,
        a: 0,
    };

    pub fn to_hex_string(&self) -> String {
        hex::encode([self.r, self.g, self.b, self.a])
    }

    pub fn is_opaque(&self) -> bool {
        self.a == u8::MAX
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.strip_prefix('#').unwrap_or(s);
        let v: Vec<u8> = if v.len() == 3 || v.len() == 4 {
            v.chars().flat_map(|it| [it as u8, it as u8]).collect()
        } else {
            v.chars().map(|it| it as u8).collect()
        };

        let bytes = hex::decode(v.as_slice()).map_err(Error::InvalidColorEncoding)?;

        Ok(Color {
            r: *bytes.first().ok_or(Error::InvalidColor {
                reason: "hex too short",
            })?,
            g: *bytes.get(1).ok_or(Error::InvalidColor {
                reason: "hex too short",
            })?,
            b: *bytes.get(2).ok_or(Error::InvalidColor {
                reason: "hex too short",
            })?,
            a: bytes.get(3).cloned().unwrap_or(u8::MAX),
        })
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields {
            Red,
            R,
            Green,
            G,
            Blue,
            B,
            Alpha,
            A,
        }

        struct ColorVisitor;
        impl<'de> de::Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("RGBA color")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Color::from_str(v).map_err(E::custom)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Color::from_str(&v).map_err(E::custom)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                if let Some(size) = seq.size_hint() {
                    if size != 3 && size != 4 {
                        return Err(de::Error::invalid_length(size, &self));
                    }
                }

                let r = seq.next_element()?.ok_or(de::Error::missing_field("r"))?;
                let g = seq.next_element()?.ok_or(de::Error::missing_field("g"))?;
                let b = seq.next_element()?.ok_or(de::Error::missing_field("b"))?;
                let a = seq.next_element()?.unwrap_or(u8::MAX);

                Ok(Color { r, g, b, a })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut r = None;
                let mut g = None;
                let mut b = None;
                let mut a = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Red | Fields::R => {
                            if r.is_some() {
                                return Err(de::Error::duplicate_field("r"));
                            }
                            r = Some(map.next_value()?);
                        }
                        Fields::Green | Fields::G => {
                            if g.is_some() {
                                return Err(de::Error::duplicate_field("g"));
                            }
                            g = Some(map.next_value()?);
                        }
                        Fields::Blue | Fields::B => {
                            if b.is_some() {
                                return Err(de::Error::duplicate_field("b"));
                            }
                            b = Some(map.next_value()?);
                        }
                        Fields::Alpha | Fields::A => {
                            if a.is_some() {
                                return Err(de::Error::duplicate_field("a"));
                            }
                            a = Some(map.next_value()?);
                        }
                    }
                }

                Ok(Color {
                    r: r.ok_or_else(|| de::Error::missing_field("r"))?,
                    g: g.ok_or_else(|| de::Error::missing_field("g"))?,
                    b: b.ok_or_else(|| de::Error::missing_field("b"))?,
                    a: a.unwrap_or(u8::MAX),
                })
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::Color;
    use crate::{error::Error, test_support::from_cbor};
    use ciborium::cbor;
    use std::str::FromStr;

    /// The four channels, as a tuple, so a whole color can be asserted at once
    /// without requiring [`Color`] to implement [`PartialEq`].
    ///
    /// Input:  `Color { r: 255, g: 136, b: 0, a: 255 }`
    /// Output: `(255, 136, 0, 255)`
    fn channels(color: &Color) -> (u8, u8, u8, u8) {
        (color.r, color.g, color.b, color.a)
    }

    #[test]
    fn six_digit_hex_parses_with_or_without_a_leading_hash() {
        let with_hash = Color::from_str("#ff8800").expect("hash prefix is optional");
        let without_hash = Color::from_str("ff8800").expect("hash prefix is optional");

        assert_eq!(channels(&with_hash), (255, 136, 0, u8::MAX));
        assert_eq!(channels(&without_hash), (255, 136, 0, u8::MAX));
    }

    #[test]
    fn eight_digit_hex_carries_the_alpha_channel() {
        let color = Color::from_str("#ff880080").expect("RRGGBBAA is accepted");
        assert_eq!(channels(&color), (255, 136, 0, 128));
    }

    /// CSS style shorthand: every digit stands for a whole channel byte.
    #[test]
    fn three_and_four_digit_hex_digits_are_doubled() {
        let rgb = Color::from_str("#f80").expect("RGB shorthand is accepted");
        let rgba = Color::from_str("#f808").expect("RGBA shorthand is accepted");

        assert_eq!(channels(&rgb), (255, 136, 0, u8::MAX));
        assert_eq!(channels(&rgba), (255, 136, 0, 136));
    }

    #[test]
    fn parsing_and_formatting_round_trip() {
        let color = Color::from_str("#ff880080").expect("valid hex");
        assert_eq!(color.to_hex_string(), "ff880080");
        assert_eq!(
            Color::from_str("#f80").expect("valid hex").to_hex_string(),
            "ff8800ff"
        );
    }

    /// Zint reads `fgcolour`/`bgcolour` as RRGGBB or RRGGBBAA, so the defaults
    /// have to survive formatting with their alpha intact.
    #[test]
    fn default_colors_use_the_format_zint_expects() {
        assert_eq!(Color::BLACK.to_hex_string(), "000000ff");
        assert_eq!(Color::TRANSPARENT.to_hex_string(), "ffffff00");

        assert!(Color::BLACK.is_opaque());
        assert!(!Color::TRANSPARENT.is_opaque());
        assert!(!Color::from_str("#ff880080").expect("valid hex").is_opaque());
    }

    #[test]
    fn an_odd_number_of_hex_digits_is_rejected() {
        let error = Color::from_str("#ff880").expect_err("five digits is not a color");
        assert!(
            matches!(
                error,
                Error::InvalidColorEncoding(hex::FromHexError::OddLength)
            ),
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn non_hex_characters_are_rejected() {
        let error = Color::from_str("#gg8800").expect_err("'g' is not a hex digit");
        assert!(
            matches!(
                error,
                Error::InvalidColorEncoding(hex::FromHexError::InvalidHexCharacter { c: 'g', .. })
            ),
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn a_hex_string_with_too_few_channels_is_rejected() {
        for too_short in ["", "ff"] {
            let error =
                Color::from_str(too_short).expect_err("a color needs at least three channels");
            assert!(
                matches!(
                    error,
                    Error::InvalidColor {
                        reason: "hex too short"
                    }
                ),
                "unexpected error for {too_short:?}: {error:?}"
            );
        }
    }

    /// `char as u8` truncates, so the parser must not be fed anything it could
    /// mistake for a hex digit; multi-byte input has to fail cleanly instead.
    #[test]
    fn non_ascii_input_is_rejected_rather_than_truncated() {
        for input in ["日本語", "ÿÿ", "ＦＦ８８００"] {
            assert!(
                Color::from_str(input).is_err(),
                "{input:?} should not parse as a color"
            );
        }
    }

    #[test]
    fn deserializes_from_a_hex_string() {
        let color: Color = from_cbor(cbor!("#ff880080").unwrap()).expect("hex string is a color");
        assert_eq!(channels(&color), (255, 136, 0, 128));
    }

    #[test]
    fn deserializes_from_a_channel_array() {
        let opaque: Color = from_cbor(cbor!([255, 136, 0]).unwrap()).expect("RGB array is a color");
        let translucent: Color =
            from_cbor(cbor!([255, 136, 0, 128]).unwrap()).expect("RGBA array is a color");

        assert_eq!(channels(&opaque), (255, 136, 0, u8::MAX));
        assert_eq!(channels(&translucent), (255, 136, 0, 128));
    }

    #[test]
    fn deserializes_from_a_channel_map_in_either_spelling() {
        let short: Color =
            from_cbor(cbor!({"r" => 255, "g" => 136, "b" => 0, "a" => 128}).unwrap())
                .expect("single letter channels");
        let long: Color =
            from_cbor(cbor!({"red" => 255, "green" => 136, "blue" => 0, "alpha" => 128}).unwrap())
                .expect("spelled out channels");

        assert_eq!(channels(&short), (255, 136, 0, 128));
        assert_eq!(channels(&long), (255, 136, 0, 128));
    }

    #[test]
    fn a_channel_given_twice_is_rejected() {
        let error =
            from_cbor::<Color>(cbor!({"r" => 255, "red" => 0, "g" => 136, "b" => 0}).unwrap())
                .expect_err("'r' and 'red' are the same channel");
        assert!(
            error.contains("duplicate field"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn a_map_missing_a_channel_is_rejected() {
        let error = from_cbor::<Color>(cbor!({"r" => 255, "g" => 136}).unwrap())
            .expect_err("blue is missing");
        assert!(error.contains("missing field"), "unexpected error: {error}");

        let alpha_only_is_optional: Color =
            from_cbor(cbor!({"r" => 255, "g" => 136, "b" => 0}).unwrap())
                .expect("alpha defaults to opaque");
        assert_eq!(channels(&alpha_only_is_optional), (255, 136, 0, u8::MAX));
    }

    /// What a document gets back when the color is not one of the shapes the
    /// manual describes.
    #[test]
    fn a_value_that_is_not_a_color_at_all_is_rejected() {
        let error = from_cbor::<Color>(cbor!(true).unwrap()).expect_err("a flag is not a color");
        assert!(
            error.contains("expected RGBA color"),
            "the error should say what was expected: {error}"
        );
    }

    #[test]
    fn an_unknown_channel_name_is_rejected() {
        let error = from_cbor::<Color>(
            cbor!({"r" => 255, "g" => 136, "b" => 0, "luminance" => 1}).unwrap(),
        )
        .expect_err("'luminance' is not a channel");
        assert!(!error.is_empty(), "the error should name the problem");
    }

    #[test]
    fn an_array_that_is_not_three_or_four_channels_is_rejected() {
        assert!(
            from_cbor::<Color>(cbor!([255, 136]).unwrap()).is_err(),
            "two channels is not a color"
        );
        assert!(
            from_cbor::<Color>(cbor!([255, 136, 0, 128, 0]).unwrap()).is_err(),
            "five channels is not a color"
        );
    }
}
