use std::{
    ffi::{CStr, CString},
    ops::{Deref, DerefMut},
};

use zint_wasm_sys::{
    free_svg_plot_string, svg_plot_string, zint_symbol, ZBarcode_Encode_and_Buffer_Vector,
};

use crate::{
    error::{Error, ZintResult},
    options::{color::Color, Options},
};

#[repr(transparent)]
pub struct Symbol {
    inner: *mut zint_symbol,
}

impl Symbol {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(options: &Options) -> Self {
        let mut result = Self::default();

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
        let result = ZintResult::from_code(unsafe {
            ZBarcode_Encode_and_Buffer_Vector(
                self.inner,
                c_str_data.as_bytes_with_nul().as_ptr(),
                length,
                rotate_angle,
            ) as u32
        });
        if let Some(err) = result.as_error() {
            return Err(Error::Zint(err));
        }
        let (result, svg) = unsafe {
            let mut result: i32 = 0;
            let svg_cstr = svg_plot_string(self.inner, &mut result);
            let svg = CStr::from_ptr(svg_cstr)
                .to_str()
                .map_err(Error::InvalidResultSVG)?
                .to_string();
            free_svg_plot_string(svg_cstr);
            (ZintResult::from_code(result as u32), svg)
        };

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
