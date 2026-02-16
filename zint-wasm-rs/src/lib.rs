mod color;
pub mod error;
pub mod options;
pub mod output;
pub mod segment;
pub mod symbol;

pub use color::Color;
pub use options::symbology::Symbology;
pub use options::{DisplayOptions, GenericOptions, SymbolOptions};
pub use symbol::Symbol;

pub(crate) mod util {
    use std::ffi::CString;

    /// Copies `src` Rust string into a `dest` C char buffer.
    ///
    /// Implementation is similar to [`copy_from_slice`] except it doesn't panic
    /// if destination isn't the same size as source.
    ///
    /// # Panics
    ///
    /// Panics if the destination buffer isn't large enough to contain the source string.
    pub fn copy_into_cstr<S: AsRef<str>>(src: S, dest: &mut [::std::os::raw::c_char]) {
        let s = CString::new(src.as_ref()).unwrap();
        let bytes = s.as_bytes_with_nul();
        // SAFETY: c_char and u8 have the same size and alignment;
        // we're just reinterpreting the byte slice as c_char slice.
        let src: &[::std::ffi::c_char] =
            unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const _, bytes.len()) };
        if dest.len() < src.len() {
            panic!("target buffer too small")
        }
        for (i, v) in src.iter().enumerate() {
            dest[i] = *v
        }
    }
}
