use std::str::FromStr;

use serde::Deserialize;

use crate::error::ZintError;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn to_hex_string(&self) -> String {
        hex::encode([self.r, self.g, self.b, self.a])
    }

    pub fn is_opaque(&self) -> bool {
        self.a == u8::MAX
    }
}

impl FromStr for Color {
    type Err = ZintError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.strip_prefix('#').unwrap_or(s);
        let v: Vec<u8> = if v.len() == 3 || v.len() == 4 {
            v.chars().flat_map(|it| [it as u8, it as u8]).collect()
        } else {
            v.chars().map(|it| it as u8).collect()
        };

        let bytes = hex::decode(v.as_slice()).map_err(|_| ZintError::InvalidColor)?;

        Ok(Color {
            r: *bytes.first().ok_or(ZintError::InvalidColor)?,
            g: *bytes.get(1).ok_or(ZintError::InvalidColor)?,
            b: *bytes.get(2).ok_or(ZintError::InvalidColor)?,
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
        pub enum Fields {
            Red,
            R,
            Green,
            G,
            Blue,
            B,
            Alpha,
            A,
        }

        pub struct ColorVisitor;
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
