use crate::{
    error::{Error, ZintResult},
    output::{PlotKind, PlotResult},
};
use zint_sys::{internal, zint_symbol};

/// Default zint EMF export.
///
/// This plotting target will use default zint EMF output and store it in a byte
/// buffer.
///
/// If you need access to vector primitives, use
/// [`VectorPlot`][crate::output::VectorPlot] instead.
pub struct EmfPlot(Vec<u8>);

impl<'a> PlotResult<'a> for EmfPlot {
    const KIND: PlotKind = PlotKind::Vector;

    fn from_symbol(
        symbol: &'a mut zint_sys::zint_symbol,
        options: &crate::options::DisplayOptions,
    ) -> Result<Self, Error> {
        let symbol_ptr = symbol as *mut zint_symbol;
        let result = ZintResult::from(
            unsafe { internal::zint_emf_plot(symbol_ptr, 0) } as u32,
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

impl AsRef<[u8]> for EmfPlot {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl From<EmfPlot> for Vec<u8> {
    fn from(val: EmfPlot) -> Self {
        val.0
    }
}
impl EmfPlot {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}
