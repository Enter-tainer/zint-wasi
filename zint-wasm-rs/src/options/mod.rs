use serde::Deserialize;

use crate::symbol::Symbol;

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
    pub fg_color: Option<Color>,
    /// background color
    pub bg_color: Option<Color>,
    /// Primary message data (MaxiCode, Composite)
    pub primary: Option<String>,
    /// Symbol-specific options (see "../docs/manual.txt")
    pub option_1: Option<i32>,
    /// Symbol-specific options (see "../docs/manual.txt")
    pub option_2: Option<i32>,
    /// Symbol-specific options (see "../docs/manual.txt")
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
    pub guard_decent: Option<f32>,
}

impl Options {
    pub fn with_symbology(symbology: Symbology) -> Self {
        Self {
            symbology,
            ..Default::default()
        }
    }
}

impl Options {
    pub fn into_zint_symbol(self) -> Symbol {
        let mut sym =
            unsafe { Symbol::from_ptr(zint_wasm_sys::ZBarcode_Create().as_mut().unwrap()) };
        let inner = unsafe { sym.get_mut() };
        inner.symbology = self.symbology as i32;
        if let Some(height) = self.height {
            inner.height = height;
        }
        if let Some(scale) = self.scale {
            inner.scale = scale;
        }
        if let Some(whitespace_width) = self.whitespace_width {
            inner.whitespace_width = whitespace_width;
        }
        if let Some(whitespace_height) = self.whitespace_height {
            inner.whitespace_height = whitespace_height;
        }
        if let Some(border_width) = self.border_width {
            inner.border_width = border_width;
        }

        if let Some(output_options) = self.output_options {
            inner.output_options = output_options.into();
        }

        if let Some(fg_color) = &self.fg_color {
            crate::util::copy_into_cstr(fg_color.to_hex_string(), &mut inner.fgcolour);
        }

        if let Some(bg_color) = &self.bg_color {
            crate::util::copy_into_cstr(bg_color.to_hex_string(), &mut inner.bgcolour);
        }

        if let Some(primary) = &self.primary {
            crate::util::copy_into_cstr(primary, &mut inner.primary);
        }

        if let Some(option_1) = self.option_1 {
            inner.option_1 = option_1;
        }

        if let Some(option_2) = self.option_2 {
            inner.option_2 = option_2;
        }

        if let Some(option_3) = self.option_3 {
            inner.option_3 = option_3.as_i32();
        }

        if let Some(show_hrt) = self.show_hrt {
            inner.show_hrt = show_hrt as i32;
        }

        if let Some(input_mode) = self.input_mode {
            inner.input_mode = input_mode as i32;
        }

        if let Some(eci) = self.eci {
            inner.eci = eci;
        }

        if let Some(dot_size) = self.dot_size {
            inner.dot_size = dot_size;
        }

        if let Some(text_gap) = self.text_gap {
            inner.text_gap = text_gap;
        }

        if let Some(guard_decent) = self.guard_decent {
            inner.guard_descent = guard_decent;
        }

        sym
    }
}
