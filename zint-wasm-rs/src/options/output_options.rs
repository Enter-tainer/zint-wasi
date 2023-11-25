use serde::{Deserialize, Serialize};
use zint_wasm_sys::*;
/// Output options
#[derive(Debug, Serialize, Deserialize)]
pub enum OutputOption {
    /// Boundary bar above the symbol only (not below), does not affect stacking
    BarcodeBindTop,
    /// Boundary bars above & below the symbol and between stacked symbols
    BarcodeBind,
    /// Box around symbol
    BarcodeBox,
    /// Output to stdout
    BarcodeStdout,
    /// Reader Initialisation (Programming)
    ReaderInit,
    /// Use smaller font
    SmallText,
    /// Use bold font
    BoldText,
    /// CMYK colour space (Encapsulated PostScript and TIF)
    CmykColour,
    /// Plot a matrix symbol using dots rather than squares
    BarcodeDottyMode,
    /// Use GS instead of FNC1 as GS1 separator (Data Matrix)
    Gs1GsSeparator,
    /// Return ASCII values in bitmap buffer (OUT_BUFFER only)
    OutBufferIntermediate,
    /// Add compliant quiet zones (additional to any specified whitespace)
    BarcodeQuietZones,
    /// Disable quiet zones, notably those with defaults as listed above
    BarcodeNoQuietZones,
    /// Warn if height not compliant, or use standard height (if any) as default
    CompliantHeight,
    /// Add quiet zone indicators ("<"/">") to HRT whitespace (EAN/UPC)
    EanUpcGuardWhitespace,
    /// Embed font in vector output - currently only for SVG output
    EmbedVectorFont,
}

impl From<OutputOption> for i32 {
    fn from(output_options: OutputOption) -> Self {
        match output_options {
            OutputOption::BarcodeBindTop => BARCODE_BIND_TOP,
            OutputOption::BarcodeBind => BARCODE_BIND,
            OutputOption::BarcodeBox => BARCODE_BOX,
            OutputOption::BarcodeStdout => BARCODE_STDOUT,
            OutputOption::ReaderInit => READER_INIT,
            OutputOption::SmallText => SMALL_TEXT,
            OutputOption::BoldText => BOLD_TEXT,
            OutputOption::CmykColour => CMYK_COLOUR,
            OutputOption::BarcodeDottyMode => BARCODE_DOTTY_MODE,
            OutputOption::Gs1GsSeparator => GS1_GS_SEPARATOR,
            OutputOption::OutBufferIntermediate => OUT_BUFFER_INTERMEDIATE,
            OutputOption::BarcodeQuietZones => BARCODE_QUIET_ZONES,
            OutputOption::BarcodeNoQuietZones => BARCODE_NO_QUIET_ZONES,
            OutputOption::CompliantHeight => COMPLIANT_HEIGHT,
            OutputOption::EanUpcGuardWhitespace => EANUPC_GUARD_WHITESPACE,
            OutputOption::EmbedVectorFont => EMBED_VECTOR_FONT,
        }
        .try_into()
        .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputOptions {
    pub options: Vec<OutputOption>,
}

impl From<OutputOptions> for i32 {
    fn from(output_options: OutputOptions) -> Self {
        let mut options: i32 = 0;
        for option in output_options.options {
            let option: i32 = option.into();
            options |= option;
        }
        options
    }
}
