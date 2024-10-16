use std::{fmt::Display, str::Utf8Error, mem::MaybeUninit};

use serde::Deserialize;
use zint_wasm_sys::*;

/// Warning conditions (API return values)
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[non_exhaustive]
pub enum ZintWarning {
    /// Human Readable Text was truncated (max 199 bytes)
    #[error("Human Readable Text was truncated (max 199 bytes)")]
    HRTTruncated = ZINT_WARN_HRT_TRUNCATED,
    /// Invalid option given but overridden by Zint
    #[error("Invalid option given but overridden by Zint")]
    InvalidOption = ZINT_WARN_INVALID_OPTION,
    /// Automatic ECI inserted by Zint
    #[error("Automatic ECI inserted by Zint")]
    UsesECI = ZINT_WARN_USES_ECI,
    /// Symbol created not compliant with standards
    #[error("Symbol created not compliant with standards")]
    Noncompliant = ZINT_WARN_NONCOMPLIANT,
}

impl ZintWarning {
    const LAST: u32 = ZINT_WARN_NONCOMPLIANT;

    /// This function isn't unsafe because `ZintWarning` is `non_exhaustive`.
    pub fn from_code(code: u32) -> Self {
        unsafe { std::mem::transmute(code) }
    }
}

/// Error conditions (API return values)
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32, C)]
#[non_exhaustive]
pub enum ZintError {
    /// Unknown error
    #[error("unknown zint error: #{0}")]
    Other(u32) = 0,
    /// Input data wrong length
    #[error("input data wrong length")]
    TooLong = ZINT_ERROR_TOO_LONG,
    /// Input data incorrect
    #[error("input data incorrect")]
    InvalidData = ZINT_ERROR_INVALID_DATA,
    /// Input check digit incorrect
    #[error("input check digit incorrect")]
    InvalidCheck = ZINT_ERROR_INVALID_CHECK,
    /// Incorrect option given
    #[error("incorrect option given")]
    InvalidOption = ZINT_ERROR_INVALID_OPTION,
    /// Internal error (should not happen)
    #[error("internal error")]
    EncodingProblem = ZINT_ERROR_ENCODING_PROBLEM,
    /// Error opening output file
    #[error("error opening output file")]
    FileAccess = ZINT_ERROR_FILE_ACCESS,
    /// Memory allocation (malloc) failure
    #[error("memory allocation failure")]
    Memory = ZINT_ERROR_MEMORY,
    /// Error writing to output file
    #[error("error writing to output file")]
    FileWrite = ZINT_ERROR_FILE_WRITE,

    // Errors caused by warnings
    /// Automatic ECI inserted by Zint
    #[error("Automatic ECI inserted by Zint")]
    UsesECI = ZINT_ERROR_USES_ECI,
    /// Symbol created not compliant with standards
    #[error("Symbol created not compliant with standards")]
    Noncompliant = ZINT_ERROR_NONCOMPLIANT,
    /// Human Readable Text was truncated (max 199 bytes)
    #[error("Human Readable Text was truncated (max 199 bytes)")]
    HRTTruncated = ZINT_ERROR_HRT_TRUNCATED,
}

impl ZintError {
    const FIRST: u32 = ZINT_ERROR_TOO_LONG;
    const LAST: u32 = ZINT_ERROR_HRT_TRUNCATED;
}

impl From<u32> for ZintError {
    /// Returns an error value from error code.
    fn from(code: u32) -> Self {
        if (Self::FIRST..=Self::LAST).contains(&code) {
            unsafe {
                // Safety: disciminant is first, explicitly declared as u32
                // padding bytes don't have to be set to 0, so setting the
                // discriminant byte to one of supported error codes and keeping
                // garbage after is fine.
                let mut result = MaybeUninit::uninit();
                let discriminant = result.as_mut_ptr() as *mut u32;
                discriminant.write(code);
                // result is now safe to read and valid
                result.assume_init()
            }
        } else {
            Self::Other(code)
        }
    }
}
impl From<ZintError> for u32 {
    /// Returns error code from error value.
    fn from(error: ZintError) -> Self {
        match error {
            ZintError::Other(code) => code,
            known => unsafe {
                // Safety: discriminant IS the error code and is the first u32
                let data = std::ptr::addr_of!(known) as *const u32;
                data.read()
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ZintOk {
    /// Ok result, indicating no errors
    Ok = 0,
}

/// A safe wrapper around Zint return values.
///
/// All variants are `u32`.
#[derive(Clone, Copy)]
pub union ZintResult {
    any: u32,
    #[allow(dead_code)]
    ok: ZintOk,
    warning: ZintWarning,
    error: ZintError,
}

impl ZintResult {
    pub fn from_code(code: u32) -> Self {
        ZintResult { any: code }
    }
    pub fn is_ok(&self) -> bool {
        unsafe {
            // Safety: any is always valid
            self.any == 0
        }
    }
    pub fn is_warning(&self) -> bool {
        unsafe {
            // Safety: any is always valid
            self.any != 0 && self.any <= ZintWarning::LAST
        }
    }
    pub fn is_error(&self) -> bool {
        unsafe {
            // Safety: any is always valid
            self.any > ZintWarning::LAST && self.any <= ZintError::LAST
        }
    }

    pub fn as_warning(&self) -> Option<ZintWarning> {
        if !self.is_warning() {
            return None;
        }
        Some(unsafe {
            // Safety: variant checked
            self.warning
        })
    }
    pub fn as_error(&self) -> Option<ZintError> {
        if !self.is_error() {
            return None;
        }
        Some(unsafe {
            // Safety: variant checked
            self.error
        })
    }
}

/// Additional information about reason for failiure.
#[derive(Debug)]
pub enum ValidationFailiure {
    // generic
    TooBig,
    Negative,
    // specific
    UnknownFormat,
    MultipleFormats,
}
impl Display for ValidationFailiure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ValidationFailiure::TooBig => "value is too large",
            ValidationFailiure::Negative => "value is negative",
            ValidationFailiure::UnknownFormat => "unknown input format",
            ValidationFailiure::MultipleFormats => "selected multiple input formats",
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error originating from Zint
    #[error(transparent)]
    Zint(#[from] ZintError),
    #[error("zint returned non-utf8 SVG result")]
    InvalidResultSVG(#[source] Utf8Error),
    /// Invalid output options
    #[error("invalid input mode: {0}")]
    InvalidInputMode(ValidationFailiure),
    /// Multiple input modes selected
    #[error("multiple input modes selected")]
    MultipleInputModes,
    /// Unknown output option
    #[error("unknown input option: {0}")]
    UnknownInputOption(String),
    /// Invalid output options
    #[error("invalid output options: {0}")]
    InvalidOutputOptions(ValidationFailiure),
    /// Unknown output option
    #[error("unknown output option: {0}")]
    UnknownOutputOption(String),
    /// Invalid color hex
    #[error("invalid color hex: {0}")]
    InvalidColorEncoding(#[source] hex::FromHexError),
    /// Invalid color format
    #[error("invalid color format; {reason}")]
    InvalidColor { reason: &'static str },
    #[error("invalid option value for {which}: {value:?}")]
    UnknownOption {
        which: &'static str,
        value: Box<dyn std::fmt::Debug>,
    },
}

/// Warning level (symbol->warn_level)
#[derive(Debug, Copy, Clone, Deserialize)]
#[repr(u32)]
pub enum WarningLevel {
    /// Default behaviour
    Default = WARN_DEFAULT,
    /// Treat warning as error
    FailAll = WARN_FAIL_ALL,
}
