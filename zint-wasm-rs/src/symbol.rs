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
        options::{
            color::Color,
            input_mode::InputMode,
            option3::{Option3, QRMatrixOption},
            output_options::OutputOptions,
            symbology::Symbology,
            Options,
        },
    };
    use std::{ffi::CStr, os::raw::c_char, str::FromStr};

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

    /// Reads one of the symbol's fixed size C string fields back as Rust.
    ///
    /// Input:  `symbol.fgcolour`, holding `000000ff\0`
    /// Output: `"000000ff"`
    fn c_string(field: &[c_char]) -> String {
        let value = unsafe {
            // Safety: every field this is used on is written by
            // `copy_into_cstr`, which always terminates, or zeroed by zint.
            CStr::from_ptr(field.as_ptr())
        };
        value.to_string_lossy().into_owned()
    }

    fn code128() -> Options {
        Options::with_symbology(Symbology::Code128)
    }

    /// Listing every field rather than using `..Default::default()` is
    /// deliberate: a new option cannot be added to [`Options`] without this
    /// test being made to say where it lands in the symbol.
    #[test]
    fn every_option_reaches_the_zint_symbol() {
        let options = Options {
            symbology: Symbology::QRCode,
            height: Some(30.0),
            scale: Some(2.0),
            whitespace_width: Some(4),
            whitespace_height: Some(2),
            border_width: Some(3),
            output_options: Some(OutputOptions::BARCODE_BOX),
            fg_color: Some(Color::from_str("#112233").expect("valid hex")),
            bg_color: Some(Color::from_str("#44556677").expect("valid hex")),
            primary: Some("331234567890".to_string()),
            option_1: Some(2),
            option_2: Some(5),
            option_3: Some(Option3::from(QRMatrixOption::FULL_MULITIBYTE)),
            show_hrt: Some(false),
            input_mode: Some(InputMode::GS1 | InputMode::ESCAPE),
            eci: Some(26),
            dot_size: Some(0.75),
            text_gap: Some(1.5),
            guard_descent: Some(4.0),
        };

        let symbol = Symbol::new(&options);

        assert_eq!(symbol.symbology, Symbology::QRCode as i32);
        assert_eq!(symbol.height, 30.0);
        assert_eq!(symbol.scale, 2.0);
        assert_eq!(symbol.whitespace_width, 4);
        assert_eq!(symbol.whitespace_height, 2);
        assert_eq!(symbol.border_width, 3);
        assert_eq!(
            symbol.output_options,
            (OutputOptions::BARCODE_BOX | OutputOptions::BARCODE_MEMORY_FILE).as_i32()
        );
        assert_eq!(c_string(&symbol.fgcolour), "112233ff");
        assert_eq!(c_string(&symbol.bgcolour), "44556677");
        assert_eq!(c_string(&symbol.primary), "331234567890");
        assert_eq!(symbol.option_1, 2);
        assert_eq!(symbol.option_2, 5);
        assert_eq!(symbol.option_3, 200);
        assert_eq!(symbol.show_hrt, 0);
        assert_eq!(
            symbol.input_mode,
            (InputMode::GS1 | InputMode::ESCAPE).as_i32()
        );
        assert_eq!(symbol.eci, 26);
        assert_eq!(symbol.dot_size, 0.75);
        assert_eq!(symbol.text_gap, 1.5);
        assert_eq!(symbol.guard_descent, 4.0);
    }

    /// Anything the caller leaves out stays at the value zint picked, so its
    /// defaults remain the single source of truth.
    #[test]
    fn unset_options_keep_the_defaults_zint_chose() {
        let symbol = Symbol::new(&code128());

        assert_eq!(symbol.height, 0.0);
        assert_eq!(symbol.scale, 1.0);
        assert_eq!(symbol.whitespace_width, 0);
        assert_eq!(symbol.whitespace_height, 0);
        assert_eq!(symbol.border_width, 0);
        assert_eq!(symbol.option_1, -1);
        assert_eq!(symbol.option_2, 0);
        assert_eq!(symbol.option_3, 0);
        assert_eq!(symbol.show_hrt, 1);
        assert_eq!(symbol.input_mode, 0);
        assert_eq!(symbol.eci, 0);
        assert_eq!(symbol.dot_size, 0.8);
        assert_eq!(symbol.text_gap, 1.0);
        assert_eq!(symbol.guard_descent, 5.0);
        assert_eq!(c_string(&symbol.primary), "");
    }

    /// The plugin has no file system to write to, so the symbol always renders
    /// into memory no matter what else the caller asked for.
    #[test]
    fn the_symbol_always_renders_into_memory() {
        let bare = Symbol::new(&code128());
        assert_eq!(
            bare.output_options,
            OutputOptions::BARCODE_MEMORY_FILE.as_i32()
        );

        let mut options = code128();
        options.output_options = Some(OutputOptions::BARCODE_BIND);
        let configured = Symbol::new(&options);
        assert_eq!(
            configured.output_options,
            (OutputOptions::BARCODE_BIND | OutputOptions::BARCODE_MEMORY_FILE).as_i32()
        );
    }

    /// Zint reads the colors as hex text; leaving them out has to give the
    /// black on transparent the Typst package documents, not zint's own white
    /// background.
    #[test]
    fn colors_default_to_black_on_transparent() {
        let symbol = Symbol::new(&code128());

        assert_eq!(c_string(&symbol.fgcolour), "000000ff");
        assert_eq!(c_string(&symbol.bgcolour), "ffffff00");
    }

    #[test]
    fn code128_encodes_to_svg_with_human_readable_text() {
        let svg = Symbol::new(&code128())
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
        let mut options = code128();
        options.scale = Some(2.0);
        let scaled = Symbol::new(&options)
            .encode_svg("A12345B", 0, 0)
            .expect("Code128 encodes at double scale");
        assert_eq!(svg_size(&scaled), (448, 233));

        let mut options = code128();
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

    /// The colors the caller chose have to survive into the drawing itself, not
    /// only into the symbol's fields.
    #[test]
    fn the_chosen_colors_are_drawn() {
        let mut options = code128();
        options.fg_color = Some(Color::from_str("#112233").expect("valid hex"));
        options.bg_color = Some(Color::from_str("#ddeeff").expect("valid hex"));

        let svg = Symbol::new(&options)
            .encode_svg("A12345B", 0, 0)
            .expect("Code128 encodes in color");

        assert!(
            svg.contains("#112233"),
            "the bars keep the foreground color"
        );
        assert!(
            svg.contains("#DDEEFF") || svg.contains("#ddeeff"),
            "an opaque background is drawn"
        );
    }

    /// Turning the symbol on its side swaps what the drawing measures.
    #[test]
    fn a_rotated_symbol_swaps_its_dimensions() {
        let (width, height) = svg_size(
            &Symbol::new(&code128())
                .encode_svg("A12345B", 0, 0)
                .expect("Code128 encodes upright"),
        );
        let turned = Symbol::new(&code128())
            .encode_svg("A12345B", 0, 90)
            .expect("Code128 encodes rotated");

        assert_eq!(svg_size(&turned), (height, width));
    }

    /// Zero means "everything up to the terminator"; any other length is the
    /// number of bytes zint reads, which is how binary payloads are bounded.
    #[test]
    fn an_explicit_length_bounds_what_is_encoded() {
        let whole = Symbol::new(&code128())
            .encode_svg("A12345B", 0, 0)
            .expect("Code128 encodes the whole payload");
        let clipped = Symbol::new(&code128())
            .encode_svg("A12345B", 3, 0)
            .expect("Code128 encodes the first three bytes");

        assert!(whole.lines().any(|line| line.trim() == "A12345B"));
        assert!(clipped.lines().any(|line| line.trim() == "A12"));
    }

    /// A warning is not a failure: zint still produces the symbol, and a
    /// document that asked for a barcode still gets one.
    #[test]
    fn a_symbol_zint_only_warns_about_is_still_returned() {
        let mut options = code128();
        options.height = Some(1.0);
        options.output_options = Some(OutputOptions::COMPLIANT_HEIGHT);

        let svg = Symbol::new(&options)
            .encode_svg("A12345B", 0, 0)
            .expect("a symbol that is too short is still a symbol");

        assert!(svg.contains("<path d=\"M"));
    }

    #[test]
    #[should_panic(expected = "can't create a Symbol from null pointer")]
    fn wrapping_a_null_pointer_panics() {
        unsafe {
            // Safety: the null case is the one `from_ptr` is documented to
            // reject, which is what this asserts.
            Symbol::from_ptr(std::ptr::null_mut())
        };
    }

    #[test]
    fn a_symbol_survives_being_handed_around_as_a_pointer() {
        let symbol = Symbol::new(&Options::with_symbology(Symbology::QRCode));
        let pointer = symbol.as_ptr();
        // Ownership moves to the raw pointer, so the symbol is not freed twice.
        std::mem::forget(symbol);

        let symbol = unsafe {
            // Safety: the pointer came from the symbol leaked just above, so it
            // still points at an initialised zint_symbol.
            Symbol::from_ptr(pointer)
        };

        assert_eq!(symbol.as_ptr(), pointer);
        assert_eq!(symbol.symbology, Symbology::QRCode as i32);
    }
}
