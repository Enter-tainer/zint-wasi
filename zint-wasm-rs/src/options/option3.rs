use std::fmt::Debug;

use serde::Deserialize;
use zint_wasm_sys::*;

use crate::error::Error;

/// Data Matrix specific options
#[derive(Debug, Clone, Copy, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
#[serde(untagged, try_from = "u32")]
#[repr(u32)]
pub enum DataMatrixOption {
    /// Only consider square versions on automatic symbol size selection
    Square = DM_SQUARE,
    /// Consider DMRE versions on automatic symbol size selection
    DMRE = DM_DMRE,
    /// Use ISO instead of "de facto" format for 144x144 (i.e. don't skew ECC)
    ISO144 = DM_ISO_144,
}

impl TryFrom<u32> for DataMatrixOption {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            DM_SQUARE => Self::Square,
            DM_DMRE => Self::DMRE,
            DM_ISO_144 => Self::ISO144,
            other => {
                return Err(Error::UnknownOption {
                    which: "option_3",
                    value: Box::new(other),
                })
            }
        })
    }
}

/// QR, Han Xin, Grid Matrix specific options
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged, try_from = "u32")]
#[repr(u32)]
#[non_exhaustive]
pub enum QRMatrixOption {
    /// Enable Kanji/Hanzi compression for Latin-1 & binary data
    FullMultibyte = ZINT_FULL_MULTIBYTE,
}

impl TryFrom<u32> for QRMatrixOption {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            ZINT_FULL_MULTIBYTE => Ok(Self::FullMultibyte),
            mask if (mask >> 8) <= 7
                && (mask & 0xFF == ZINT_FULL_MULTIBYTE || mask & 0xFF == 0) =>
            {
                Ok(unsafe { std::mem::transmute::<u32, QRMatrixOption>(mask) })
            }
            _ => Err(Error::UnknownOption {
                which: "option_3",
                value: Box::new(value),
            }),
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

impl TryFrom<u32> for Option3 {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        // don't care which variant it is, we're just checking that the value
        // can be stored as one
        DataMatrixOption::try_from(value)
            .map(From::<DataMatrixOption>::from)
            .or_else(|_| QRMatrixOption::try_from(value).map(From::<QRMatrixOption>::from))
            .or_else(|_| UltracodeOption::try_from(value).map(From::<UltracodeOption>::from))
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
                formatter.write_str("option_3 value")
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
                    "dm-square" | "square" => Option3::from(DataMatrixOption::Square),
                    "dm-dmre" | "dmre" | "rect" => Option3::from(DataMatrixOption::DMRE),
                    "dm-iso-144" | "iso-144" => Option3::from(DataMatrixOption::ISO144),
                    "zint-full-multibyte" | "full-multibyte" => {
                        Option3::from(QRMatrixOption::FullMultibyte)
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
