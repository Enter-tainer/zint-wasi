use std::{fmt::Display, mem::MaybeUninit, str::Utf8Error};

use serde::Deserialize;
use zint_wasm_sys::*;

macro_rules! in_range_or_other {
    ($owner: ident, $repr: ty) => {
        impl From<$repr> for $owner {
            /// Returns a warning value from warning code.
            fn from(code: $repr) -> Self {
                if (Self::FIRST..=Self::LAST).contains(&code) {
                    unsafe {
                        // Safety: disciminant is first, explicitly declared as $repr
                        // padding bytes don't have to be set to 0, so setting the
                        // discriminant byte to one of supported error codes and keeping
                        // garbage after is fine.
                        let mut result = MaybeUninit::uninit();
                        let discriminant = result.as_mut_ptr() as *mut $repr;
                        discriminant.write(code);
                        // result is now safe to read and valid
                        result.assume_init()
                    }
                } else {
                    Self::Other(code)
                }
            }
        }
        impl From<$owner> for $repr {
            /// Returns warning code from warning value.
            fn from(error: $owner) -> Self {
                match error {
                    $owner::Other(code) => code,
                    known => unsafe {
                        // Safety: discriminant IS the code and is the first $repr
                        let data = std::ptr::addr_of!(known) as *const $repr;
                        data.read()
                    },
                }
            }
        }
    };
}

/// Warning conditions (API return values)
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32, C)]
#[non_exhaustive]
pub enum ZintWarning {
    /// Unknown warning
    #[error("unknown zint warning: #{0}")]
    Other(u32),
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
    const FIRST: u32 = ZINT_WARN_HRT_TRUNCATED;
    const LAST: u32 = ZINT_WARN_NONCOMPLIANT;
}
in_range_or_other!(ZintWarning, u32);

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
    const FIRST: u32 = ZintWarning::LAST + 1;
    const LAST: u32 = ZINT_ERROR_HRT_TRUNCATED;
}
in_range_or_other!(ZintError, u32);

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
pub struct ZintResult(u32);
impl ZintResult {
    pub fn is_ok(&self) -> bool {
        self.0 == 0
    }
    pub fn is_warning(&self) -> bool {
        (ZintWarning::FIRST..=ZintWarning::LAST).contains(&self.0)
    }
    pub fn is_error(&self) -> bool {
        (ZintError::FIRST..).contains(&self.0)
    }
    pub fn as_warning(&self) -> Option<ZintWarning> {
        if !self.is_warning() {
            return None;
        }
        Some(ZintWarning::from(self.0))
    }
    pub fn as_error(&self) -> Option<ZintError> {
        if !self.is_error() {
            return None;
        }
        Some(ZintError::from(self.0))
    }
}
impl From<u32> for ZintResult {
    #[inline]
    fn from(value: u32) -> Self {
        ZintResult(value)
    }
}
impl From<ZintResult> for u32 {
    #[inline]
    fn from(value: ZintResult) -> Self {
        value.0
    }
}

impl From<ZintOk> for ZintResult {
    #[inline]
    fn from(_: ZintOk) -> Self {
        ZintResult(0)
    }
}
impl From<ZintWarning> for ZintResult {
    #[inline]
    fn from(warning: ZintWarning) -> Self {
        ZintResult(warning.into())
    }
}
impl From<ZintError> for ZintResult {
    #[inline]
    fn from(error: ZintError) -> Self {
        ZintResult(error.into())
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
