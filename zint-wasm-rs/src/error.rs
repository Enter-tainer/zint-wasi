use std::{ffi::CStr, fmt::Display, mem::MaybeUninit};

use crate::options::symbology::SymbolOptionError;
use zint_sys::*;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32, C)]
#[non_exhaustive]
pub enum ZintWarningKind {
    /// Unknown warning
    Other(u32),
    /// Human Readable Text was truncated (max 199 bytes)
    HRTTruncated = ZINT_WARN_HRT_TRUNCATED,
    /// Invalid option given but overridden by Zint
    InvalidOption = ZINT_WARN_INVALID_OPTION,
    /// Automatic ECI inserted by Zint
    UsesECI = ZINT_WARN_USES_ECI,
    /// Symbol created not compliant with standards
    Noncompliant = ZINT_WARN_NONCOMPLIANT,
}
impl ZintWarningKind {
    const FIRST: u32 = ZINT_WARN_HRT_TRUNCATED;
    const LAST: u32 = ZINT_WARN_NONCOMPLIANT;
}
in_range_or_other!(ZintWarningKind, u32);

impl Display for ZintWarningKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ZintWarningKind::Other(number) => return write!(f, "unknown zint warning: #{number}"),
            ZintWarningKind::HRTTruncated => "Human Readable Text was truncated (max 199 bytes)",
            ZintWarningKind::InvalidOption => "Invalid option given but overridden by Zint",
            ZintWarningKind::UsesECI => "Automatic ECI inserted by Zint",
            ZintWarningKind::Noncompliant => "Symbol created not compliant with standards",
        })
    }
}

#[derive(Debug, Clone)]
pub struct ZintWarning {
    kind: ZintWarningKind,
    message: Option<String>,
}

impl Display for ZintWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = &self.message {
            f.write_str(message)
        } else {
            Display::fmt(&self.kind, f)
        }
    }
}

/// Error conditions (API return values)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32, C)]
#[non_exhaustive]
pub enum ZintErrorKind {
    /// Unknown error
    Other(u32) = 0,
    /// Input data wrong length
    TooLong = ZINT_ERROR_TOO_LONG,
    /// Input data incorrect
    InvalidData = ZINT_ERROR_INVALID_DATA,
    /// Input check digit incorrect
    InvalidCheck = ZINT_ERROR_INVALID_CHECK,
    /// Incorrect option given
    InvalidOption = ZINT_ERROR_INVALID_OPTION,
    /// Internal error (should not happen)
    EncodingProblem = ZINT_ERROR_ENCODING_PROBLEM,
    /// Error opening output file
    FileAccess = ZINT_ERROR_FILE_ACCESS,
    /// Memory allocation (malloc) failure
    Memory = ZINT_ERROR_MEMORY,
    /// Error writing to output file
    FileWrite = ZINT_ERROR_FILE_WRITE,

    // Errors caused by warnings
    /// Automatic ECI inserted by Zint
    UsesECI = ZINT_ERROR_USES_ECI,
    /// Symbol created not compliant with standards
    NotCompliant = ZINT_ERROR_NONCOMPLIANT,
    /// Human Readable Text was truncated (max 199 bytes)
    HRTTruncated = ZINT_ERROR_HRT_TRUNCATED,
}
impl ZintErrorKind {
    const FIRST: u32 = ZintWarningKind::LAST + 1;
    const LAST: u32 = ZINT_ERROR_HRT_TRUNCATED;
}
in_range_or_other!(ZintErrorKind, u32);

impl Display for ZintErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Other(other) => return write!(f, "unknown zint error: #{other}"),
            Self::TooLong => "input data wrong length",
            Self::InvalidData => "input data incorrect",
            Self::InvalidCheck => "input check digit incorrect",
            Self::InvalidOption => "incorrect option given",
            Self::EncodingProblem => "internal error",
            Self::FileAccess => "error opening output file",
            Self::Memory => "memory allocation failure",
            Self::FileWrite => "error writing to output file",
            Self::UsesECI => "Automatic ECI inserted by Zint",
            Self::NotCompliant => "created Symbol is not compliant with standards",
            Self::HRTTruncated => "Human Readable Text was truncated (max 199 bytes)",
        })
    }
}

#[derive(Debug, Clone)]
pub struct ZintError {
    kind: ZintErrorKind,
    message: Option<String>,
}
impl Display for ZintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = &self.message {
            f.write_str(message)
        } else {
            Display::fmt(&self.kind, f)
        }
    }
}
impl std::error::Error for ZintError {}

/// A safe wrapper around zint return values.
///
/// zint always returns a single warning/error if the result isn't `Ok`
/// (indicated by 0), with errors having precedence over warnings. In other
/// words, if both an error and warning occur during some procedure, only the
/// error will be returned.
#[derive(Clone)]
pub enum ZintResult {
    Ok,
    Warning(ZintWarning),
    Error(ZintError),
}
impl ZintResult {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn from(value: u32, symbol: *const zint_symbol) -> Self {
        if value == 0 {
            ZintResult::Ok
        } else {
            if symbol.is_null() {
                return ZintResult::Error(ZintError {
                    kind: ZintErrorKind::InvalidData,
                    message: Some("symbol is null".to_string()),
                });
            }
            let symbol = unsafe { symbol.as_ref().unwrap_unchecked() };
            let message = unsafe {
                let error_text = std::mem::transmute::<&[i8], &[u8]>(&symbol.errtxt);
                CStr::from_bytes_until_nul(error_text).unwrap_unchecked()
            };
            let message = Some(message.to_string_lossy().to_string());
            if value <= ZintWarningKind::LAST {
                let kind = ZintWarningKind::from(value);
                ZintResult::Warning(ZintWarning { kind, message })
            } else {
                let kind = ZintErrorKind::from(value);
                ZintResult::Error(ZintError { kind, message })
            }
        }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self, ZintResult::Ok)
    }
    pub fn is_warning(&self) -> bool {
        matches!(self, ZintResult::Warning(_))
    }
    pub fn is_error(&self) -> bool {
        matches!(self, ZintResult::Error(_))
    }
    pub fn as_warning(&self) -> Option<&ZintWarning> {
        match self {
            ZintResult::Warning(zint_warning) => Some(zint_warning),
            _ => None,
        }
    }
    pub fn as_error(&self) -> Option<&ZintError> {
        match self {
            ZintResult::Error(zint_error) => Some(zint_error),
            _ => None,
        }
    }
}

/// Additional information about reason for failure.
#[derive(Debug)]
pub enum ValidationFailure {
    // generic
    TooBig,
    Negative,
    // specific
    UnknownFormat,
    MultipleFormats,
}

impl Display for ValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ValidationFailure::TooBig => "value is too large",
            ValidationFailure::Negative => "value is negative",
            ValidationFailure::UnknownFormat => "unknown input format",
            ValidationFailure::MultipleFormats => "selected multiple input formats",
        })
    }
}

#[derive(Debug)]
pub enum ZintProblem {
    Error(ZintError),
    Warning(ZintWarning),
}
impl Display for ZintProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZintProblem::Error(zint_error) => Display::fmt(zint_error, f),
            ZintProblem::Warning(zint_warning) => Display::fmt(zint_warning, f),
        }
    }
}
impl std::error::Error for ZintProblem {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error originating from Zint
    #[error(transparent)]
    Zint(#[from] ZintProblem),
    /// Invalid output options
    #[error("invalid input mode: {0}")]
    InvalidInputMode(ValidationFailure),
    /// Multiple input modes selected
    #[error("multiple input modes selected")]
    MultipleInputModes,
    /// Unknown output option
    #[error("unknown input option: {0}")]
    UnknownInputOption(String),
    /// Invalid output options
    #[error("invalid output options: {0}")]
    InvalidOutputOptions(ValidationFailure),
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
    #[error(transparent)]
    SymbolOptionError(#[from] SymbolOptionError),
    /// Non-latin character found in input for `encode_ascii`
    #[error("non-latin character '{}' at position {position} in encode_ascii input", .char)]
    NonLatinCharacter { char: char, position: usize },
    /// Too many segments provided for the symbology
    #[error("too many segments: provided {provided}, maximum allowed {maximum}")]
    TooManySegments { provided: usize, maximum: usize },
    /// This error occurs when trying to plot a Symbol that hasn't been
    /// encoded with vector output enabled.
    #[error("Symbol is missing vector data")]
    MissingVectorData,
    /// This error occurs when trying to plot a Symbol that hasn't been
    /// encoded with raster output enabled.
    #[error("Symbol is missing raster data")]
    MissingRasterData,
}

impl From<ZintError> for Error {
    fn from(value: ZintError) -> Self {
        Self::Zint(ZintProblem::Error(value))
    }
}

impl From<ZintWarning> for Error {
    fn from(value: ZintWarning) -> Self {
        Self::Zint(ZintProblem::Warning(value))
    }
}
