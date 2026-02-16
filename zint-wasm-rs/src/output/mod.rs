use crate::options::DisplayOptions;

#[cfg(feature = "raster")]
pub mod raster;

#[cfg(feature = "emf")]
mod emf;
#[cfg(feature = "ps")]
mod ps;
#[cfg(feature = "svg")]
mod svg;
#[cfg(feature = "vector")]
pub mod vector;

#[cfg(feature = "emf")]
pub use emf::EmfPlot;
#[cfg(feature = "ps")]
pub use ps::PsPlot;
#[cfg(feature = "svg")]
pub use svg::SvgPlot;
#[cfg(feature = "vector")]
pub use vector::VectorPlot;

pub enum PlotKind {
    Raster,
    Vector,
}

pub trait PlotResult<'a>: Sized {
    /// Communicates whether this result requires vector or raster data input to
    /// be populated in the symbol.
    const KIND: PlotKind;

    fn from_symbol(
        symbol: &'a mut zint_sys::zint_symbol,
        options: &DisplayOptions,
    ) -> Result<Self, crate::error::Error>;
}
