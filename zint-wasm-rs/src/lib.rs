pub mod error;
pub mod options;
pub mod symbol;

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
        let src: &[::std::os::raw::c_char] = unsafe {
            // Safety: C string is a sequence of c_chars
            std::mem::transmute(s.as_bytes_with_nul())
        };
        if dest.len() < src.len() {
            panic!("target buffer too small")
        }
        for (i, v) in src.iter().enumerate() {
            dest[i] = *v
        }
    }
}
