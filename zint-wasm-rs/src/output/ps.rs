use crate::{
    error::{Error, ZintResult},
    output::{PlotKind, PlotResult},
};
use std::fmt::Display;
use zint_sys::{internal, zint_symbol};

/// Default zint Postscript export.
///
/// This plotting target will use default zint PS output and store it in a byte
/// buffer.
///
/// If you need access to vector primitives, use
/// [`VectorPlot`][crate::output::VectorPlot] instead.
pub struct PsPlot(Vec<u8>);

impl<'a> PlotResult<'a> for PsPlot {
    const KIND: PlotKind = PlotKind::Vector;

    fn from_symbol(
        symbol: &'a mut zint_sys::zint_symbol,
        options: &crate::options::DisplayOptions,
    ) -> Result<Self, Error> {
        let symbol_ptr = symbol as *mut zint_symbol;
        let result = ZintResult::from(
            unsafe { internal::zint_ps_plot(symbol_ptr, 0) } as u32,
            symbol_ptr,
        );
        match result {
            ZintResult::Warning(zint_warning) => match options.warnings {
                crate::options::WarningHandling::Ignore => {}
                crate::options::WarningHandling::LogWarnings(level) => {
                    log::log!(level, "{zint_warning}");
                }
                crate::options::WarningHandling::AsErrors => {
                    return Err(Error::from(zint_warning));
                }
            },
            ZintResult::Error(zint_error) => {
                return Err(Error::from(zint_error));
            }
            ZintResult::Ok => {}
        }
        let out_buffer =
            unsafe { std::slice::from_raw_parts(symbol.memfile, symbol.memfile_size as usize) };
        Ok(Self(out_buffer.to_vec()))
    }
}

impl AsRef<str> for PsPlot {
    fn as_ref(&self) -> &str {
        // SAFETY: zint produces UTF-8 output for PostScript
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}
impl From<PsPlot> for Vec<u8> {
    fn from(value: PsPlot) -> Self {
        value.0
    }
}
impl From<PsPlot> for String {
    fn from(value: PsPlot) -> Self {
        value.as_ref().to_string()
    }
}

impl Display for PsPlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
