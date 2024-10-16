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
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let mask = QRMatrixOption::from_bits_truncate(value);
        match mask {
            invalid if invalid.bits() != value => Err(Error::UnknownOption {
                which: "option_3",
                value: Box::new(value),
            }),
            valid => Ok(valid),
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
