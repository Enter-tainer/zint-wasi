use serde::{Deserialize, Serialize};
use zint_wasm_sys::*;
/// Input modes
#[derive(Debug, Serialize, Deserialize)]
pub enum InputMode {
    /// Binary
    Data,
    /// UTF-8
    Unicode,
    /// GS1
    Gs1,
    /// Process escape sequences
    Escape,
    /// Process parentheses as GS1 AI delimiters (instead of square brackets)
    Gs1Parens,
    /// Do not check validity of GS1 data (except that printable ASCII only)
    Gs1NoCheck,
    /// Interpret `height` as per-row rather than as overall height
    HeightPerRow,
    /// Use faster if less optimal encodation or other shortcuts if available
    Fast,
    /// Process special symbology-specific escape sequences
    ExtraEscape,
}

impl From<InputMode> for i32 {
    fn from(input_mode: InputMode) -> Self {
        match input_mode {
            InputMode::Data => DATA_MODE,
            InputMode::Unicode => UNICODE_MODE,
            InputMode::Gs1 => GS1_MODE,
            InputMode::Escape => ESCAPE_MODE,
            InputMode::Gs1Parens => GS1PARENS_MODE,
            InputMode::Gs1NoCheck => GS1NOCHECK_MODE,
            InputMode::HeightPerRow => HEIGHTPERROW_MODE,
            InputMode::Fast => FAST_MODE,
            InputMode::ExtraEscape => EXTRA_ESCAPE_MODE,
        }
        .try_into()
        .unwrap()
    }
}
