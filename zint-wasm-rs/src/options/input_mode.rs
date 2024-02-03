use serde::Deserialize;
use zint_wasm_sys::*;

use crate::error::ZintError;

/// Input modes
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[repr(i32)]
pub enum InputMode {
    /// Binary
    Data = DATA_MODE as i32,
    /// UTF-8
    Unicode = UNICODE_MODE as i32,
    /// GS1
    Gs1 = GS1_MODE as i32,
    /// Process escape sequences
    Escape = ESCAPE_MODE as i32,
    /// Process parentheses as GS1 AI delimiters (instead of square brackets)
    Gs1Parens = GS1PARENS_MODE as i32,
    /// Do not check validity of GS1 data (except that printable ASCII only)
    Gs1NoCheck = GS1NOCHECK_MODE as i32,
    /// Interpret `height` as per-row rather than as overall height
    HeightPerRow = HEIGHTPERROW_MODE as i32,
    /// Use faster if less optimal encodation or other shortcuts if available
    Fast = FAST_MODE as i32,
    /// Process special symbology-specific escape sequences
    ExtraEscape = EXTRA_ESCAPE_MODE as i32,
}

impl TryFrom<i32> for InputMode {
    type Error = ZintError;

    fn try_from(input_mode: i32) -> Result<Self, Self::Error> {
        Ok(match input_mode as u32 {
            DATA_MODE => InputMode::Data,
            UNICODE_MODE => InputMode::Unicode,
            GS1_MODE => InputMode::Gs1,
            ESCAPE_MODE => InputMode::Escape,
            GS1PARENS_MODE => InputMode::Gs1Parens,
            GS1NOCHECK_MODE => InputMode::Gs1NoCheck,
            HEIGHTPERROW_MODE => InputMode::HeightPerRow,
            FAST_MODE => InputMode::Fast,
            EXTRA_ESCAPE_MODE => InputMode::ExtraEscape,
            _ => return Err(ZintError::InvalidOption),
        })
    }
}
