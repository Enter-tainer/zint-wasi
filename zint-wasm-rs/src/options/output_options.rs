use serde::{Deserialize, Serialize};
use zint_wasm_sys::*;
/// Output options
#[derive(Debug, Serialize, Deserialize)]
pub enum OutputOptions {
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

impl From<OutputOptions> for u32 {
    fn from(output_options: OutputOptions) -> Self {
        match output_options {
            OutputOptions::BarcodeBindTop => BARCODE_BIND_TOP,
            OutputOptions::BarcodeBind => BARCODE_BIND,
            OutputOptions::BarcodeBox => BARCODE_BOX,
            OutputOptions::BarcodeStdout => BARCODE_STDOUT,
            OutputOptions::ReaderInit => READER_INIT,
            OutputOptions::SmallText => SMALL_TEXT,
            OutputOptions::BoldText => BOLD_TEXT,
            OutputOptions::CmykColour => CMYK_COLOUR,
            OutputOptions::BarcodeDottyMode => BARCODE_DOTTY_MODE,
            OutputOptions::Gs1GsSeparator => GS1_GS_SEPARATOR,
            OutputOptions::OutBufferIntermediate => OUT_BUFFER_INTERMEDIATE,
            OutputOptions::BarcodeQuietZones => BARCODE_QUIET_ZONES,
            OutputOptions::BarcodeNoQuietZones => BARCODE_NO_QUIET_ZONES,
            OutputOptions::CompliantHeight => COMPLIANT_HEIGHT,
            OutputOptions::EanUpcGuardWhitespace => EANUPC_GUARD_WHITESPACE,
            OutputOptions::EmbedVectorFont => EMBED_VECTOR_FONT,
        }
    }
}
