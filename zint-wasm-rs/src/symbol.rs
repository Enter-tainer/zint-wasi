use zint_sys::*;

use crate::{
    error::{Error, ZintResult},
    options::{
        DisplayOptions, GenericOptions, output_options::OutputOptions, symbology::Symbology,
    },
    output::PlotResult,
    segment::{ECI, Segment},
};

#[repr(transparent)]
pub struct Symbol {
    inner: *mut zint_symbol,
}

#[inline(always)]
fn make_zint_symbol(symbology: Symbology) -> *mut zint_symbol {
    let result_ptr = unsafe { zint_sys::ZBarcode_Create() };
    let result = unsafe { result_ptr.as_mut().expect("ZBarcode_Create returned null") };
    result.symbology = symbology as i32;
    // must be set
    result.outfile[0] = '\0' as i8;
    // always output to memory
    result.output_options = OutputOptions::BARCODE_MEMORY_FILE.as_c_int();

    if cfg!(test) {
        result.debug = zint_sys::ZINT_DEBUG_PRINT as i32;
    }
    result_ptr
}

impl Symbol {
    pub fn encode_ascii<'o>(
        options: impl Into<GenericOptions<'o>>,
        text: &str,
    ) -> Result<Self, Error> {
        let options = options.into();
        for (position, char) in text.chars().enumerate() {
            if char as u32 > 0xFF {
                return Err(Error::NonLatinCharacter { char, position });
            }
        }
        let eci = if options.supports_eci() {
            ECI::ISO_8859_1
        } else {
            ECI::NONE
        };
        Self::encode_segments(options, &[Segment::new(text.as_bytes(), eci)])
    }

    pub fn encode_utf8<'o>(
        options: impl Into<GenericOptions<'o>>,
        text: &str,
    ) -> Result<Self, Error> {
        let options = options.into();
        let eci = if options.supports_eci() {
            ECI::UTF_8
        } else {
            ECI::NONE
        };
        Self::encode_segments(options, &[Segment::new(text.as_bytes(), eci)])
    }

    pub fn encode_segments<'o>(
        options: impl Into<GenericOptions<'o>>,
        data: &[Segment<'_>],
    ) -> Result<Self, Error> {
        let options = options.into();

        // SAFETY: Segment is a transparent wrapper of zint_seg
        let segments: &[zint_seg] =
            unsafe { std::mem::transmute::<&[Segment<'_>], &[zint_seg]>(data) };
        let segment_count = segments.len();
        let max_segments = if options.supports_eci() {
            zint_sys::ZINT_MAX_SEG_COUNT as usize
        } else {
            1
        };
        if segment_count > max_segments {
            return Err(Error::TooManySegments {
                provided: segment_count,
                maximum: max_segments,
            });
        }
        let segments = segments.as_ptr();

        let result_ptr = make_zint_symbol(options.symbology);
        let result = unsafe { result_ptr.as_mut().unwrap_unchecked() };
        options.apply(result)?;

        let result = ZintResult::from(
            unsafe { ZBarcode_Encode_Segs(result_ptr, segments, segment_count as i32) as u32 },
            result_ptr,
        );
        match result {
            ZintResult::Error(zint_error) => return Err(Error::from(zint_error)),
            ZintResult::Warning(zint_warning) => match options.warnings {
                crate::options::WarningHandling::Ignore => {}
                crate::options::WarningHandling::LogWarnings(level) => {
                    log::log!(level, "{zint_warning}");
                }
                crate::options::WarningHandling::AsErrors => return Err(Error::from(zint_warning)),
            },
            ZintResult::Ok => {}
        }
        Ok(Self { inner: result_ptr })
    }

    fn plot_raster_data(&self, options: &DisplayOptions) -> Result<(), Error> {
        let result = ZintResult::from(
            unsafe { ZBarcode_Buffer(self.inner, options.rotation.into()) as u32 },
            self.inner,
        );

        match result {
            ZintResult::Error(zint_error) => return Err(Error::from(zint_error)),
            ZintResult::Warning(zint_warning) => match options.warnings {
                crate::options::WarningHandling::Ignore => {}
                crate::options::WarningHandling::LogWarnings(level) => {
                    log::log!(level, "{zint_warning}");
                }
                crate::options::WarningHandling::AsErrors => return Err(Error::from(zint_warning)),
            },
            ZintResult::Ok => {}
        }
        Ok(())
    }

    fn plot_vector_data(&self, options: &DisplayOptions) -> Result<(), Error> {
        let result = ZintResult::from(
            unsafe { ZBarcode_Buffer_Vector(self.inner, options.rotation.into()) as u32 },
            self.inner,
        );

        match result {
            ZintResult::Error(zint_error) => return Err(Error::from(zint_error)),
            ZintResult::Warning(zint_warning) => match options.warnings {
                crate::options::WarningHandling::Ignore => {}
                crate::options::WarningHandling::LogWarnings(level) => {
                    log::log!(level, "{zint_warning}");
                }
                crate::options::WarningHandling::AsErrors => return Err(Error::from(zint_warning)),
            },
            ZintResult::Ok => {}
        }
        Ok(())
    }

    pub fn plot<'a, R: PlotResult<'a>>(
        &'a mut self,
        options: &'a DisplayOptions,
    ) -> Result<R, Error> {
        let result: &'a mut zint_symbol = unsafe { self.inner.as_mut().unwrap_unchecked() };
        options.apply(result)?;

        match R::KIND {
            crate::output::PlotKind::Raster => self.plot_raster_data(options)?,
            crate::output::PlotKind::Vector => self.plot_vector_data(options)?,
        }

        R::from_symbol(result, options)
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        unsafe {
            zint_sys::ZBarcode_Delete(self.inner);
        }
    }
}
