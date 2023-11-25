use serde::{Deserialize, Serialize};
use zint_wasm_sys::*;
/// Data Matrix specific options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum DataMatrixOption {
    /// Only consider square versions on automatic symbol size selection
    Square,
    /// Consider DMRE versions on automatic symbol size selection
    DMRE,
    /// Use ISO instead of "de facto" format for 144x144 (i.e. don't skew ECC)
    ISO144,
}

impl From<DataMatrixOption> for i32 {
    fn from(val: DataMatrixOption) -> Self {
        match val {
            DataMatrixOption::Square => DM_SQUARE,
            DataMatrixOption::DMRE => DM_DMRE,
            DataMatrixOption::ISO144 => DM_ISO_144,
        }
        .try_into()
        .unwrap()
    }
}

/// QR, Han Xin, Grid Matrix specific options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QRMatrixOption {
    /// Enable Kanji/Hanzi compression for Latin-1 & binary data
    FullMultibyte,
}

impl From<QRMatrixOption> for i32 {
    fn from(val: QRMatrixOption) -> Self {
        match val {
            QRMatrixOption::FullMultibyte => ZINT_FULL_MULTIBYTE,
        }
        .try_into()
        .unwrap()
    }
}

/// Ultracode specific option
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UltracodeOption {
    /// Enable Ultracode compression (experimental)
    Compression,
}

impl From<UltracodeOption> for i32 {
    fn from(val: UltracodeOption) -> Self {
        match val {
            UltracodeOption::Compression => ULTRA_COMPRESSION,
        }
        .try_into()
        .unwrap()
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Option3 {
    DataMatrix(DataMatrixOption),
    QRMatrix(QRMatrixOption),
    Ultracode(UltracodeOption),
}

impl From<Option3> for i32 {
    fn from(val: Option3) -> Self {
        match val {
            Option3::DataMatrix(val) => val.into(),
            Option3::QRMatrix(val) => val.into(),
            Option3::Ultracode(val) => val.into(),
        }
    }
}
