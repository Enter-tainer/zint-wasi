use serde::{Deserialize, Serialize};
use zint_wasm_sys::*;

/// Warning conditions (API return values)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ZintWarning {
    /// Human Readable Text was truncated (max 199 bytes)
    HRTTruncated,
    /// Invalid option given but overridden by Zint
    InvalidOption,
    /// Automatic ECI inserted by Zint
    UsesECI,
    /// Symbol created not compliant with standards
    Noncompliant,
}

/// Error conditions (API return values)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ZintError {
    /// Input data wrong length
    TooLong,
    /// Input data incorrect
    InvalidData,
    /// Input check digit incorrect
    InvalidCheck,
    /// Incorrect option given
    InvalidOption,
    /// Internal error (should not happen)
    EncodingProblem,
    /// Error opening output file
    FileAccess,
    /// Memory allocation (malloc) failure
    Memory,
    /// Error writing to output file
    FileWrite,
    /// Error counterpart of warning if WARN_FAIL_ALL set (see below)
    UsesECI,
    /// Error counterpart of warning if WARN_FAIL_ALL set
    Noncompliant,
    /// Error counterpart of warning if WARN_FAIL_ALL set
    HRTTruncated,
}

/// Warning level (symbol->warn_level)
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WarningLevel {
    /// Default behaviour
    Default,
    /// Treat warning as error
    FailAll,
}

impl From<WarningLevel> for i32 {
    fn from(val: WarningLevel) -> Self {
        match val {
            WarningLevel::Default => WARN_DEFAULT,
            WarningLevel::FailAll => WARN_FAIL_ALL,
        }
        .try_into()
        .unwrap()
    }
}

impl From<ZintWarning> for i32 {
    fn from(val: ZintWarning) -> Self {
        match val {
            ZintWarning::HRTTruncated => ZINT_WARN_HRT_TRUNCATED,
            ZintWarning::InvalidOption => ZINT_WARN_INVALID_OPTION,
            ZintWarning::UsesECI => ZINT_WARN_USES_ECI,
            ZintWarning::Noncompliant => ZINT_WARN_NONCOMPLIANT,
        }
        .try_into()
        .unwrap()
    }
}

impl ZintWarning {
    fn max_value() -> i32 {
        ZINT_WARN_NONCOMPLIANT.try_into().unwrap()
    }
}

impl From<i32> for ZintWarning {
    fn from(val: i32) -> Self {
        match val.try_into().unwrap() {
            ZINT_WARN_HRT_TRUNCATED => ZintWarning::HRTTruncated,
            ZINT_WARN_INVALID_OPTION => ZintWarning::InvalidOption,
            ZINT_WARN_USES_ECI => ZintWarning::UsesECI,
            ZINT_WARN_NONCOMPLIANT => ZintWarning::Noncompliant,
            _ => panic!("Unknown warning code: {}", val),
        }
    }
}

impl From<ZintError> for i32 {
    fn from(val: ZintError) -> Self {
        match val {
            ZintError::TooLong => ZINT_ERROR_TOO_LONG,
            ZintError::InvalidData => ZINT_ERROR_INVALID_DATA,
            ZintError::InvalidCheck => ZINT_ERROR_INVALID_CHECK,
            ZintError::InvalidOption => ZINT_ERROR_INVALID_OPTION,
            ZintError::EncodingProblem => ZINT_ERROR_ENCODING_PROBLEM,
            ZintError::FileAccess => ZINT_ERROR_FILE_ACCESS,
            ZintError::Memory => ZINT_ERROR_MEMORY,
            ZintError::FileWrite => ZINT_ERROR_FILE_WRITE,
            ZintError::UsesECI => ZINT_ERROR_USES_ECI,
            ZintError::Noncompliant => ZINT_ERROR_NONCOMPLIANT,
            ZintError::HRTTruncated => ZINT_ERROR_HRT_TRUNCATED,
        }
        .try_into()
        .unwrap()
    }
}

impl From<i32> for ZintError {
    fn from(val: i32) -> Self {
        match val.try_into().unwrap() {
            ZINT_ERROR_TOO_LONG => ZintError::TooLong,
            ZINT_ERROR_INVALID_DATA => ZintError::InvalidData,
            ZINT_ERROR_INVALID_CHECK => ZintError::InvalidCheck,
            ZINT_ERROR_INVALID_OPTION => ZintError::InvalidOption,
            ZINT_ERROR_ENCODING_PROBLEM => ZintError::EncodingProblem,
            ZINT_ERROR_FILE_ACCESS => ZintError::FileAccess,
            ZINT_ERROR_MEMORY => ZintError::Memory,
            ZINT_ERROR_FILE_WRITE => ZintError::FileWrite,
            ZINT_ERROR_USES_ECI => ZintError::UsesECI,
            ZINT_ERROR_NONCOMPLIANT => ZintError::Noncompliant,
            ZINT_ERROR_HRT_TRUNCATED => ZintError::HRTTruncated,
            _ => panic!("Unknown error code: {}", val),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ZintErrorWarning {
    Error(ZintError),
    Warning(ZintWarning),
}

impl From<ZintErrorWarning> for i32 {
    fn from(val: ZintErrorWarning) -> Self {
        match val {
            ZintErrorWarning::Error(err) => err.into(),
            ZintErrorWarning::Warning(warn) => warn.into(),
        }
    }
}

impl From<i32> for ZintErrorWarning {
    fn from(val: i32) -> Self {
        if val <= ZintWarning::max_value() {
            ZintErrorWarning::Warning(val.into())
        } else {
            ZintErrorWarning::Error(val.into())
        }
    }
}
