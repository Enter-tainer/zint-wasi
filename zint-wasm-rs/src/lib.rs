pub mod error;
pub mod options;
pub mod symbol;

#[cfg(test)]
mod test_support;

pub(crate) mod util {
    use crate::error::Error;

    /// Copies `src` into a `dest` C char buffer that zint reads as a C string.
    ///
    /// `field` names the option in the error, because the caller is the only
    /// one that knows which of zint's fixed size fields is being written.
    ///
    /// Input:  `("primary", "331234567890", &mut symbol.primary)`
    /// Output: the first 13 bytes of `symbol.primary` are the digits and a
    ///         terminator, and the rest is left as it was
    pub fn copy_into_cstr(
        field: &'static str,
        src: &str,
        dest: &mut [::std::os::raw::c_char],
    ) -> Result<(), Error> {
        let src = src.as_bytes();
        if let Some(position) = src.iter().position(|byte| *byte == 0) {
            return Err(Error::ValueContainsNul { field, position });
        }
        // The terminator needs room of its own.
        if src.len() >= dest.len() {
            return Err(Error::ValueTooLong {
                field,
                length: src.len(),
                capacity: dest.len(),
            });
        }
        for (slot, byte) in dest.iter_mut().zip(src.iter().chain(std::iter::once(&0))) {
            *slot = *byte as ::std::os::raw::c_char;
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::copy_into_cstr;
        use crate::error::Error;
        use std::os::raw::c_char;

        /// Zint reads these buffers as C strings, so the terminator matters as
        /// much as the payload.
        #[test]
        fn copies_the_string_and_its_terminator() {
            let mut buffer = [0 as c_char; 4];
            copy_into_cstr("field", "abc", &mut buffer[..]).expect("three bytes and a terminator");

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
            copy_into_cstr("field", "ab", &mut buffer[..]).expect("two bytes and a terminator");

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
            copy_into_cstr("field", "ab", &mut buffer[..]).expect("two bytes and a terminator");
            assert_eq!(buffer, [b'a' as c_char, b'b' as c_char, 0 as c_char]);
        }

        #[test]
        fn accepts_an_empty_string() {
            let mut buffer = [b'x' as c_char; 2];
            copy_into_cstr("field", "", &mut buffer[..]).expect("just a terminator");
            assert_eq!(buffer, [0 as c_char, b'x' as c_char]);
        }

        /// One byte short, because the terminator needs room too.
        #[test]
        fn reports_a_value_that_does_not_fit() {
            let mut buffer = [0 as c_char; 3];
            let error = copy_into_cstr("primary", "abc", &mut buffer[..])
                .expect_err("three bytes and a terminator need four");

            assert!(
                matches!(
                    error,
                    Error::ValueTooLong {
                        field: "primary",
                        length: 3,
                        capacity: 3
                    }
                ),
                "unexpected error: {error:?}"
            );
        }

        /// Zint would read the value up to the NUL and silently drop the rest,
        /// so the caller hears about it instead.
        #[test]
        fn reports_a_value_with_a_nul_inside_it() {
            let mut buffer = [0 as c_char; 8];
            let error = copy_into_cstr("primary", "ab\0cd", &mut buffer[..])
                .expect_err("a C string cannot carry a NUL");

            assert!(
                matches!(
                    error,
                    Error::ValueContainsNul {
                        field: "primary",
                        position: 2
                    }
                ),
                "unexpected error: {error:?}"
            );
        }
    }
}
