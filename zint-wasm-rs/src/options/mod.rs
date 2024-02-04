use serde::Deserialize;

use self::{
    color::Color, input_mode::InputMode, option3::Option3, output_options::OutputOptions,
    symbology::Symbology,
};

pub mod capability;
pub mod color;
pub mod input_mode;
pub mod option3;
pub mod output_options;
pub mod symbology;

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "typst", serde(rename_all = "kebab-case"))]
#[serde(default)]
pub struct Options {
    /// Barcode symbol to use
    #[serde(flatten)]
    pub symbology: Symbology,
    /// Barcode height in X-dimensions (ignored for fixed-width barcodes)
    pub height: Option<f32>,
    /// Scale factor when printing barcode, i.e. adjusts X-dimension. Default 1
    pub scale: Option<f32>,
    /// Width in X-dimensions of whitespace to left & right of barcode
    pub whitespace_width: Option<i32>,
    /// Height in X-dimensions of whitespace above & below the barcode
    pub whitespace_height: Option<i32>,
    /// Size of border in X-dimensions
    pub border_width: Option<i32>,
    /// Various output parameters (bind, box etc, see below)
    pub output_options: Option<OutputOptions>,
    /// foreground color
    #[serde(alias = "fg_colour")]
    #[cfg_attr(feature = "typst", serde(alias = "stroke"))]
    pub fg_color: Option<Color>,
    /// background color
    #[serde(alias = "bg_colour")]
    #[cfg_attr(feature = "typst", serde(alias = "fill"))]
    pub bg_color: Option<Color>,
    /// Primary message data (MaxiCode, Composite)
    pub primary: Option<String>,
    /// Symbol-specific options
    pub option_1: Option<i32>,
    /// Symbol-specific options
    pub option_2: Option<i32>,
    /// Symbol-specific options
    pub option_3: Option<Option3>,
    /// Show (1) or hide (0) Human Readable Text (HRT). Default 1
    pub show_hrt: Option<bool>,
    /// Encoding of input data
    pub input_mode: Option<InputMode>,
    /// Extended Channel Interpretation.
    pub eci: Option<i32>,
    /// Size of dots used in BARCODE_DOTTY_MODE.
    pub dot_size: Option<f32>,
    /// Gap between barcode and text (HRT) in X-dimensions.
    pub text_gap: Option<f32>,
    /// Height in X-dimensions that EAN/UPC guard bars descend.
    pub guard_descent: Option<f32>,
}

impl Options {
    pub fn with_symbology(symbology: Symbology) -> Self {
        Self {
            symbology,
            ..Default::default()
        }
    }
}
