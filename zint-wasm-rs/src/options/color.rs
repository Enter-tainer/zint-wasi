use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "untagged")]
pub enum Color {
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: u8 },
}

impl Color {
    pub fn to_hex_string(&self) -> String {
        match self {
            Color::Rgb { r, g, b } => {
                format!("{:02X}{:02X}{:02X}", r, g, b)
            }
            Color::Rgba { r, g, b, a } => {
                format!("{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
            }
        }
    }
}
