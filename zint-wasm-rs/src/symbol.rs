use std::{
    ffi::CString,
    ops::{Deref, DerefMut},
};

use zint_wasm_sys::{zint_symbol, ZBarcode_Encode_and_Print};

use crate::{
    error::{Error, ZintResult},
    options::{color::Color, output_options, Options},
};

#[repr(transparent)]
pub struct Symbol {
    inner: *mut zint_symbol,
}

impl Symbol {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(options: &Options) -> Self {
        let mut result = Self::default();
        let filename = "res.svg";

        result.symbology = options.symbology as i32;

        if let Some(height) = options.height {
            result.height = height;
        }
        if let Some(scale) = options.scale {
            result.scale = scale;
        }
        if let Some(whitespace_width) = options.whitespace_width {
            result.whitespace_width = whitespace_width;
        }
        if let Some(whitespace_height) = options.whitespace_height {
            result.whitespace_height = whitespace_height;
        }
        if let Some(border_width) = options.border_width {
            result.border_width = border_width;
        }

        if let Some(output_options) = options.output_options {
            result.output_options = output_options.as_i32();
        }
        // Always write to memory file
        result.output_options |= output_options::OutputOptions::BARCODE_MEMORY_FILE.as_i32();

        crate::util::copy_into_cstr(
            options.fg_color.unwrap_or(Color::BLACK).to_hex_string(),
            &mut result.fgcolour,
        );

        crate::util::copy_into_cstr(
            options
                .bg_color
                .unwrap_or(Color::TRANSPARENT)
                .to_hex_string(),
            &mut result.bgcolour,
        );

        crate::util::copy_into_cstr(filename, &mut result.outfile);

        if let Some(ref primary) = options.primary {
            crate::util::copy_into_cstr(primary, &mut result.primary);
        }

        if let Some(option_1) = options.option_1 {
            result.option_1 = option_1;
        }

        if let Some(option_2) = options.option_2 {
            result.option_2 = option_2;
        }

        if let Some(option_3) = options.option_3 {
            result.option_3 = option_3.as_i32();
        }

        if let Some(show_hrt) = options.show_hrt {
            result.show_hrt = show_hrt as i32;
        }

        if let Some(ref input_mode) = options.input_mode {
            result.input_mode = input_mode.as_i32();
        }

        if let Some(eci) = options.eci {
            result.eci = eci;
        }

        if let Some(dot_size) = options.dot_size {
            result.dot_size = dot_size;
        }

        if let Some(text_gap) = options.text_gap {
            result.text_gap = text_gap;
        }

        if let Some(guard_descent) = options.guard_descent {
            result.guard_descent = guard_descent;
        }

        result
    }

    /// # Safety
    ///
    /// Provided `ptr` must point to a properly initalized `Symbol`.
    pub unsafe fn from_ptr(ptr: *mut zint_symbol) -> Self {
        if ptr.is_null() {
            panic!("can't create a Symbol from null pointer")
        }
        Self { inner: ptr }
    }

    pub fn as_ptr(&self) -> *mut zint_symbol {
        self.inner
    }

    pub fn encode_svg(self, data: &str, length: i32, rotate_angle: i32) -> Result<String, Error> {
        let c_str_data = CString::new(data).expect("CString::new failed");
        let result = ZintResult::from(unsafe {
            ZBarcode_Encode_and_Print(
                self.inner,
                c_str_data.as_bytes_with_nul().as_ptr(),
                length,
                rotate_angle,
            ) as u32
        });
        if let Some(err) = result.as_error() {
            return Err(Error::Zint(err));
        }
        let svg = unsafe {
            let memfile = std::slice::from_raw_parts(self.memfile, self.memfile_size as usize);
            memfile
        };
        let svg = String::from_utf8_lossy(svg).to_string();

        match result.as_error() {
            Some(err) => Err(Error::Zint(err)),
            None => Ok(svg),
        }
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Self {
            inner: unsafe { zint_wasm_sys::ZBarcode_Create() },
        }
    }
}

impl Deref for Symbol {
    type Target = zint_symbol;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // Safety: Symbol is always created as a valid zint_symbol
            self.inner.as_ref().unwrap()
        }
    }
}

impl DerefMut for Symbol {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // Safety: Symbol is always created as a valid zint_symbol
            self.inner.as_mut().unwrap()
        }
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        unsafe {
            zint_wasm_sys::ZBarcode_Delete(self.inner);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Symbol;
    use crate::{
        error::{Error, ZintError},
        options::{symbology::Symbology, Options},
    };

    /// Extracts the pixel dimensions from the `<svg>` element of a Zint SVG.
    ///
    /// Input:  `<svg width="224" height="117" version="1.1" ...>`
    /// Output: `(224, 117)`
    fn svg_size(svg: &str) -> (u32, u32) {
        let element = svg
            .lines()
            .find(|line| line.starts_with("<svg "))
            .expect("SVG root element");
        let attribute = |name: &str| {
            let rest = element
                .split_once(&format!("{}=\"", name))
                .unwrap_or_else(|| panic!("`{}` attribute", name))
                .1;
            rest.split_once('"')
                .expect("closing quote")
                .0
                .parse()
                .expect("numeric attribute")
        };
        (attribute("width"), attribute("height"))
    }

    #[test]
    fn code128_encodes_to_svg_with_human_readable_text() {
        let options = Options::with_symbology(Symbology::Code128);
        let svg = Symbol::new(&options)
            .encode_svg("A12345B", 0, 0)
            .expect("Code128 encodes alphanumeric data");

        assert!(svg.starts_with("<?xml version=\"1.0\""));
        assert_eq!(svg_size(&svg), (224, 117));
        // The bars themselves plus the human readable text below them.
        assert!(svg.contains("<path d=\"M"));
        assert!(svg.lines().any(|line| line.trim() == "A12345B"));
    }

    #[test]
    fn scale_and_hrt_options_are_applied() {
        let mut options = Options::with_symbology(Symbology::Code128);
        options.scale = Some(2.0);
        let scaled = Symbol::new(&options)
            .encode_svg("A12345B", 0, 0)
            .expect("Code128 encodes at double scale");
        assert_eq!(svg_size(&scaled), (448, 233));

        let mut options = Options::with_symbology(Symbology::Code128);
        options.show_hrt = Some(false);
        let bars_only = Symbol::new(&options)
            .encode_svg("A12345B", 0, 0)
            .expect("Code128 encodes without human readable text");
        assert!(!bars_only.contains("<text"));
        // Dropping the text keeps the width but removes the room reserved for it.
        assert_eq!(svg_size(&bars_only), (224, 100));
    }

    #[test]
    fn ean13_check_digit_is_validated() {
        let options = Options::with_symbology(Symbology::EANXChk);
        let svg = Symbol::new(&options)
            .encode_svg("6975004310001", 0, 0)
            .expect("EAN-13 with a correct check digit encodes");
        assert_eq!(svg_size(&svg), (226, 118));

        let options = Options::with_symbology(Symbology::EANXChk);
        let error = Symbol::new(&options)
            .encode_svg("6975004310002", 0, 0)
            .expect_err("EAN-13 with a wrong check digit is rejected");
        assert!(
            matches!(error, Error::Zint(ZintError::InvalidCheck)),
            "unexpected error: {error:?}"
        );
    }
}
