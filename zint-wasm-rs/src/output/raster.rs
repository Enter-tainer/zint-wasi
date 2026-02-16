//! Raster output allows using [`image::DynamicImage`] as a plotting target.
//!
//! [`DynamicImage`] supports various different raster output formats, depending
//! on enabled `image` crate features. This ensures larger format coverage than
//! that of zint library and integrates well with rest of the Rust ecosystem.

use crate::{
    error::Error,
    options::DisplayOptions,
    output::{PlotKind, PlotResult},
};
use image::{DynamicImage, GenericImage as _};

impl<'a> PlotResult<'a> for DynamicImage {
    const KIND: PlotKind = PlotKind::Raster;

    fn from_symbol(
        symbol: &'a mut zint_sys::zint_symbol,
        _options: &DisplayOptions,
    ) -> Result<Self, crate::error::Error> {
        let width = symbol.bitmap_width as u32;
        let height = symbol.bitmap_height as u32;
        let slice_length = (width * height) as usize;
        if slice_length == 0 {
            return Err(Error::MissingRasterData);
        }

        let bitmap = symbol.bitmap;
        if bitmap.is_null() {
            return Err(Error::MissingRasterData);
        }
        let bitmap: &[[u8; 3]] =
            unsafe { std::slice::from_raw_parts(bitmap as *const [u8; 3], slice_length) };
        let alpha = symbol.alphamap;
        if alpha.is_null() {
            return Err(Error::MissingRasterData);
        }
        let alpha = unsafe { std::slice::from_raw_parts(alpha, slice_length) };

        let mut result = DynamicImage::new_rgba8(width, height);
        for y in 0..height {
            for x in 0..width {
                let i = (x + y * width) as usize;
                let [r, g, b] = bitmap[i];
                let a = alpha[i];
                result.put_pixel(x, y, image::Rgba([r, g, b, a]));
            }
        }
        Ok(result)
    }
}
