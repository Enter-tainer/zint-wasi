use self::{input_mode::InputMode, output_options::OutputOptions};

#[cfg(feature = "display")]
use crate::color::Color;
use crate::error::Error;
use crate::options::{
    capability::CapabilityFlags,
    rotation::Rotation,
    symbology::{SymbolOptionError, Symbology},
};
use crate::segment::Segment;
use zint_sys::zint_symbol;

pub mod capability;
pub mod input_mode;
mod option3;
pub mod output_options;
pub mod rotation;
pub mod symbology;
pub mod values;

#[derive(Debug, Clone, Copy)]
pub enum WarningHandling {
    /// Ignore warnings
    Ignore,
    /// Log warnings using specified logging level
    LogWarnings(log::Level),
    /// Treat warnings as errors
    AsErrors,
}

pub use symbology::SymbolOptions;

/// Data necessary for encoding the symbol.
///
/// This struct represents a union of all options that are passed to Zint. If
/// you know what the target symbol will be at compile-time, prefer using
/// [`SymbolOptions`] instead.
#[derive(Clone)]
pub struct GenericOptions<'s> {
    /// [`Symbology`] to use for data encoding.
    pub symbology: Symbology,

    /// Barcode height in X-dimensions (ignored for fixed-width barcodes)
    ///
    /// Value must be in range `[0.0, 2000.0]`.
    pub height: f32,
    /// Width in X-dimensions of whitespace to left & right of barcode
    ///
    /// Value must be in range `[0, 100]`.
    pub whitespace_width: u32,
    /// Height in X-dimensions of whitespace above & below the barcode
    ///
    /// Value must be in range `[0, 100]`.
    pub whitespace_height: u32,
    /// Size of border in X-dimensions
    ///
    /// Value must be in range `[0, 100]`.
    pub border_width: u32,
    /// Gap between barcode and text (HRT) in X-dimensions.
    ///
    /// Value must be in range `[-5.0, 10.0]`.
    pub text_gap: f32,
    /// Height in X-dimensions that EAN/UPC guard bars descend. Default is 5.
    ///
    /// Value must be in range `[0.0, 50.0]`.
    pub guard_descent: f32,

    /// Various output parameters (bind, box, etc.)
    ///
    /// See [`OutputOptions`] for details.
    pub output_options: OutputOptions,

    /// Encoding of input data
    pub input_mode: Option<InputMode>,

    /// Size of dots used in when [`OutputOptions::BARCODE_DOTTY_MODE`] is enabled.
    ///
    /// Value must be in range `[0.01, 20.0]`.
    pub dot_size: Option<f32>,
    /// Primary message (MaxiCode, Composite)
    pub primary_message: Option<Segment<'s>>,
    /// Symbol-specific options
    pub option_1: Option<std::ffi::c_int>,
    /// Symbol-specific options
    pub option_2: Option<std::ffi::c_int>,
    /// Symbol-specific options
    pub option_3: Option<std::ffi::c_int>,

    /// Specifies how the bindings should handle warnings produced by zint
    pub warnings: WarningHandling,
}

impl<'s> Default for GenericOptions<'s> {
    fn default() -> Self {
        Self {
            symbology: Symbology::QRCode,
            height: 0.0,
            whitespace_width: 0,
            whitespace_height: 0,
            border_width: 0,
            text_gap: 1.0,
            guard_descent: 5.0,
            output_options: OutputOptions::empty(),
            input_mode: None,
            dot_size: None,
            primary_message: None,
            option_1: None,
            option_2: None,
            option_3: None,
            warnings: WarningHandling::Ignore,
        }
    }
}

impl<'s> GenericOptions<'s> {
    pub fn from_symbology(symbology: Symbology) -> Self {
        Self {
            symbology,
            ..Default::default()
        }
    }

    pub(crate) fn apply(&self, symbol: &mut zint_symbol) -> Result<(), SymbolOptionError> {
        symbol.height = self.height;
        symbol.whitespace_width = self.whitespace_width as i32;
        symbol.whitespace_height = self.whitespace_height as i32;
        symbol.border_width = self.border_width as i32;
        symbol.text_gap = self.text_gap;
        symbol.guard_descent = self.guard_descent;
        symbol.output_options |= Into::<std::ffi::c_int>::into(self.output_options);
        if let Some(input_mode) = self.input_mode {
            symbol.input_mode = input_mode.into();
        }
        if let Some(dot_size) = self.dot_size {
            symbol.dot_size = dot_size;
        }
        if let Some(primary) = &self.primary_message {
            let length = primary.write_to_cchar_buffer(&mut symbol.primary);
            symbol.primary[length.min(127)] = '\0' as i8;
        }
        if let Some(option_1) = self.option_1 {
            symbol.option_1 = option_1;
        }
        if let Some(option_2) = self.option_2 {
            symbol.option_2 = option_2;
        }
        if let Some(option_3) = self.option_3 {
            symbol.option_3 = option_3;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn supports_eci(&self) -> bool {
        self.symbology.capabilities().contains(CapabilityFlags::ECI)
    }
}

pub struct DisplayOptions {
    /// Scale factor when printing barcode, i.e. adjusts X-dimension. Default is 1.
    ///
    /// Value must be in range `[0.01, 200.0]`.
    #[cfg(feature = "display")]
    pub scale: f32,
    /// foreground color
    #[cfg(feature = "display")]
    pub foreground: Color,
    /// background color
    #[cfg(feature = "display")]
    pub background: Color,

    /// Symbol rotation
    #[cfg(feature = "display")]
    pub rotation: Rotation,

    /// Whether to show Human Readable Text (HRT)
    pub show_hrt: bool,

    /// Specifies how the bindings should handle warnings produced by zint
    pub warnings: WarningHandling,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            #[cfg(feature = "display")]
            scale: 1.0,
            #[cfg(feature = "display")]
            foreground: Color::BLACK,
            #[cfg(feature = "display")]
            background: Color::TRANSPARENT,
            #[cfg(feature = "display")]
            rotation: Rotation::Deg0,
            show_hrt: true,
            warnings: WarningHandling::Ignore,
        }
    }
}

impl DisplayOptions {
    pub(crate) fn apply(&self, symbol: &mut zint_symbol) -> Result<(), Error> {
        #[cfg(feature = "display")]
        {
            symbol.scale = self.scale;
            crate::util::copy_into_cstr(self.foreground.to_hex_string(), &mut symbol.fgcolour);
            crate::util::copy_into_cstr(self.background.to_hex_string(), &mut symbol.bgcolour);
        }
        symbol.show_hrt = self.show_hrt as i32;

        Ok(())
    }
}
