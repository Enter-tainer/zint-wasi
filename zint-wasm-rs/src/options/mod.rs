use std::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::symbol::Symbol;

use self::{
    color::Color, input_mode::InputMode, option3::Option3, output_options::OutputOptions,
    symbology::Barcode,
};

mod capability;
mod color;
mod input_mode;
mod option3;
mod output_options;
mod symbology;

#[derive(Debug, Serialize, Deserialize)]
pub struct Options {
    /// Barcode symbol to use
    symbology: Barcode,
    /// Barcode height in X-dimensions (ignored for fixed-width barcodes)
    height: Option<f32>,
    /// Scale factor when printing barcode, i.e. adjusts X-dimension. Default 1
    scale: Option<f32>,
    /// Width in X-dimensions of whitespace to left & right of barcode
    whitespace_width: Option<i32>,
    /// Height in X-dimensions of whitespace above & below the barcode
    whitespace_height: Option<i32>,
    /// Size of border in X-dimensions
    border_width: Option<i32>,
    /// Various output parameters (bind, box etc, see below)
    output_options: Option<OutputOptions>,
    /// foreground color
    fg_color: Option<Color>,
    /// background color
    bg_color: Option<Color>,
    /// Primary message data (MaxiCode, Composite)
    primary: Option<String>,
    /// Symbol-specific options (see "../docs/manual.txt")
    option_1: Option<i32>,
    /// Symbol-specific options (see "../docs/manual.txt")
    option_2: Option<i32>,
    /// Symbol-specific options (see "../docs/manual.txt")
    option_3: Option<Option3>,
    /// Show (1) or hide (0) Human Readable Text (HRT). Default 1
    show_hrt: Option<bool>,
    /// Encoding of input data
    input_mode: Option<InputMode>,
    /// Extended Channel Interpretation.
    eci: Option<i32>,
    /// Size of dots used in BARCODE_DOTTY_MODE.
    dot_size: Option<f32>,
    /// Gap between barcode and text (HRT) in X-dimensions.
    text_gap: Option<f32>,
    /// Height in X-dimensions that EAN/UPC guard bars descend.
    guard_decent: Option<f32>,
}

impl Options {
    pub unsafe fn to_zint_symbol(self) -> Symbol {
        let mut sym = Symbol::from_ptr(zint_wasm_sys::ZBarcode_Create().as_mut().unwrap());
        let inner = sym.get_mut();
        inner.symbology = self.symbology.into();
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

        if let Some(fg_color) = self.fg_color {
            let fg_color = fg_color.to_hex_string();
            let fg_color = CString::new(fg_color).expect("CString::new failed");
            let slice_u8: &[u8] = fg_color.as_bytes_with_nul();
            let slice_i8: &[i8] = unsafe {
                std::slice::from_raw_parts(slice_u8.as_ptr() as *const i8, slice_u8.len())
            };
            inner.fgcolour.copy_from_slice(slice_i8);
        }

        if let Some(bg_color) = self.bg_color {
            let bg_color = bg_color.to_hex_string();
            let bg_color = CString::new(bg_color).expect("CString::new failed");
            let slice_u8: &[u8] = bg_color.as_bytes_with_nul();
            let slice_i8: &[i8] = unsafe {
                std::slice::from_raw_parts(slice_u8.as_ptr() as *const i8, slice_u8.len())
            };
            inner.bgcolour.copy_from_slice(slice_i8);
        }

        if let Some(primary) = self.primary {
            let primary = CString::new(primary).expect("CString::new failed");
            let slice_u8: &[u8] = primary.as_bytes_with_nul();
            let slice_i8: &[i8] = unsafe {
                std::slice::from_raw_parts(slice_u8.as_ptr() as *const i8, slice_u8.len())
            };
            inner.primary.copy_from_slice(slice_i8);
        }

        if let Some(option_1) = self.option_1 {
            inner.option_1 = option_1;
        }

        if let Some(option_2) = self.option_2 {
            inner.option_2 = option_2;
        }

        if let Some(option_3) = self.option_3 {
            inner.option_3 = option_3.into();
        }

        if let Some(show_hrt) = self.show_hrt {
            inner.show_hrt = show_hrt as i32;
        }

        if let Some(input_mode) = self.input_mode {
            inner.input_mode = input_mode.into();
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
