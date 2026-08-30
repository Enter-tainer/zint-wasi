pub mod error;
pub mod options;
pub mod symbol;

#[cfg(test)]
mod test_support;

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

    #[cfg(test)]
    mod tests {
        use super::copy_into_cstr;
        use std::os::raw::c_char;

        /// Zint reads these buffers as C strings, so the terminator matters as
        /// much as the payload.
        #[test]
        fn copies_the_string_and_its_terminator() {
            let mut buffer = [0 as c_char; 4];
            copy_into_cstr("abc", &mut buffer[..]);

            assert_eq!(
                buffer,
                [b'a' as c_char, b'b' as c_char, b'c' as c_char, 0 as c_char]
            );
        }

        /// The tail is deliberately left alone: zint stops at the terminator, so
        /// a shorter value written over a longer one still reads back correctly.
        #[test]
        fn leaves_the_rest_of_the_buffer_untouched() {
            let mut buffer = [b'x' as c_char; 6];
            copy_into_cstr("ab", &mut buffer[..]);

            assert_eq!(
                buffer,
                [
                    b'a' as c_char,
                    b'b' as c_char,
                    0 as c_char,
                    b'x' as c_char,
                    b'x' as c_char,
                    b'x' as c_char
                ]
            );
        }

        #[test]
        fn accepts_a_buffer_that_fits_exactly() {
            let mut buffer = [0 as c_char; 3];
            copy_into_cstr("ab", &mut buffer[..]);
            assert_eq!(buffer, [b'a' as c_char, b'b' as c_char, 0 as c_char]);
        }

        #[test]
        fn accepts_an_empty_string() {
            let mut buffer = [b'x' as c_char; 2];
            copy_into_cstr("", &mut buffer[..]);
            assert_eq!(buffer, [0 as c_char, b'x' as c_char]);
        }

        /// One byte short, because the terminator needs room too.
        #[test]
        #[should_panic(expected = "target buffer too small")]
        fn panics_when_the_buffer_cannot_hold_the_terminator() {
            let mut buffer = [0 as c_char; 3];
            copy_into_cstr("abc", &mut buffer[..]);
        }
    }
}
