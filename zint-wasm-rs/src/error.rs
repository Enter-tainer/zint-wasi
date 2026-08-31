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
    /// An `option_3` value that names nothing zint defines, for any symbology
    #[error(
        "invalid option value for option_3: {value}; expected 100 (square), 101 (rect), \
         128 (iso-144, or Ultracode compression), 200 (full-multibyte), or a QR mask \
         between 0x100 and 0x800{hint}"
    )]
    UnknownOption3 { value: u32, hint: &'static str },
    /// A value that does not fit the fixed size field zint keeps it in
    #[error("{field} is {length} bytes, but zint keeps it in {capacity} including the terminator")]
    ValueTooLong {
        field: &'static str,
        length: usize,
        capacity: usize,
    },
    /// A value zint reads as a C string, with a NUL somewhere inside it
    #[error("{field} contains a NUL byte at position {position}")]
    ValueContainsNul {
        field: &'static str,
        position: usize,
    },
    /// A payload longer than zint can be told about
    #[error("the data is {length} bytes, more than zint can be given at once")]
    DataTooLong { length: usize },
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

#[cfg(test)]
mod tests {
    use super::{
        Error, ValidationFailiure, WarningLevel, ZintError, ZintOk, ZintResult, ZintWarning,
    };

    /// Every code zint documents, so the reader can see the whole range the
    /// classification below is drawn from.
    const WARNINGS: &[(u32, ZintWarning)] = &[
        (1, ZintWarning::HRTTruncated),
        (2, ZintWarning::InvalidOption),
        (3, ZintWarning::UsesECI),
        (4, ZintWarning::Noncompliant),
    ];
    const ERRORS: &[(u32, ZintError)] = &[
        (5, ZintError::TooLong),
        (6, ZintError::InvalidData),
        (7, ZintError::InvalidCheck),
        (8, ZintError::InvalidOption),
        (9, ZintError::EncodingProblem),
        (10, ZintError::FileAccess),
        (11, ZintError::Memory),
        (12, ZintError::FileWrite),
        (13, ZintError::UsesECI),
        (14, ZintError::Noncompliant),
        (15, ZintError::HRTTruncated),
    ];

    /// The conversion writes the return code into the discriminant of a
    /// partially uninitialised value, so it is worth proving on every code that
    /// what comes back is the variant that code stands for.
    #[test]
    fn every_documented_code_converts_to_its_variant_and_back() {
        for (code, expected) in WARNINGS {
            assert_eq!(ZintWarning::from(*code), *expected, "code {code}");
            assert_eq!(u32::from(*expected), *code, "{expected:?}");
        }

        for (code, expected) in ERRORS {
            assert_eq!(ZintError::from(*code), *expected, "code {code}");
            assert_eq!(u32::from(*expected), *code, "{expected:?}");
        }
    }

    /// A newer zint can return a code this build has never heard of; it has to
    /// survive the round trip rather than be mistaken for a known one.
    #[test]
    fn a_code_outside_the_known_range_is_kept_as_it_is() {
        for code in [0, 5, 16, 100, u32::MAX] {
            assert_eq!(ZintWarning::from(code), ZintWarning::Other(code));
            assert_eq!(u32::from(ZintWarning::Other(code)), code);
        }

        for code in [0, 1, 4, 16, 100, u32::MAX] {
            assert_eq!(ZintError::from(code), ZintError::Other(code));
            assert_eq!(u32::from(ZintError::Other(code)), code);
        }
    }

    /// Zero is success, 1 to 4 are warnings and everything above is an error;
    /// these are the boundaries either side of each of those steps.
    #[test]
    fn return_codes_are_classified_at_the_boundaries_zint_documents() {
        let ok = ZintResult::from(0);
        assert!(ok.is_ok() && !ok.is_warning() && !ok.is_error());
        assert!(ok.as_warning().is_none() && ok.as_error().is_none());

        for code in [1, 4] {
            let result = ZintResult::from(code);
            assert!(result.is_warning(), "{code} is a warning");
            assert!(
                !result.is_ok() && !result.is_error(),
                "{code} is only a warning"
            );
            assert!(result.as_warning().is_some() && result.as_error().is_none());
        }

        for code in [5, 15, 16, u32::MAX] {
            let result = ZintResult::from(code);
            assert!(result.is_error(), "{code} is an error");
            assert!(
                !result.is_ok() && !result.is_warning(),
                "{code} is only an error"
            );
            assert!(result.as_error().is_some() && result.as_warning().is_none());
        }
    }

    /// Codes above the documented range still have to be reported as errors,
    /// not silently dropped, so that a newer zint cannot return a failure this
    /// wrapper treats as success.
    #[test]
    fn an_unknown_error_code_is_still_an_error() {
        let result = ZintResult::from(16);

        assert!(result.is_error());
        assert_eq!(result.as_error(), Some(ZintError::Other(16)));
    }

    #[test]
    fn results_convert_from_and_to_the_raw_code() {
        assert_eq!(u32::from(ZintResult::from(ZintOk::Ok)), 0);
        assert_eq!(u32::from(ZintResult::from(ZintWarning::Noncompliant)), 4);
        assert_eq!(u32::from(ZintResult::from(ZintError::InvalidCheck)), 7);
        assert_eq!(u32::from(ZintResult::from(ZintError::Other(99))), 99);
    }

    /// An unknown code is the one message that has to carry a number, since
    /// there is nothing else to go on when reporting it.
    #[test]
    fn unknown_codes_keep_their_number_in_the_message() {
        assert!(ZintWarning::Other(42).to_string().contains("42"));
        assert!(ZintError::Other(42).to_string().contains("42"));
    }

    #[test]
    fn validation_failures_explain_themselves() {
        assert_eq!(ValidationFailiure::TooBig.to_string(), "value is too large");
        assert_eq!(
            ValidationFailiure::Negative.to_string(),
            "value is negative"
        );
        assert_eq!(
            ValidationFailiure::UnknownFormat.to_string(),
            "unknown input format"
        );
        assert_eq!(
            ValidationFailiure::MultipleFormats.to_string(),
            "selected multiple input formats"
        );
    }

    /// A zint failure reaches Typst as the plugin's error string, so it must
    /// not be wrapped in wording of our own.
    #[test]
    fn a_zint_error_is_reported_verbatim() {
        let error = Error::from(ZintError::InvalidCheck);

        assert_eq!(error.to_string(), ZintError::InvalidCheck.to_string());
        assert!(matches!(error, Error::Zint(ZintError::InvalidCheck)));
    }

    #[test]
    fn option_errors_name_the_option_and_the_value() {
        let error = Error::UnknownOption {
            which: "option_3",
            value: Box::new(42),
        };

        let message = error.to_string();
        assert!(
            message.contains("option_3"),
            "unexpected message: {message}"
        );
        assert!(message.contains("42"), "unexpected message: {message}");
    }

    #[test]
    fn warning_levels_match_the_values_zint_defines() {
        assert_eq!(WarningLevel::Default as u32, 0);
        assert_eq!(WarningLevel::FailAll as u32, 2);
    }
}
