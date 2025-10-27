use std::fmt::Debug;

use serde::Deserialize;
use zint_sys::*;

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
