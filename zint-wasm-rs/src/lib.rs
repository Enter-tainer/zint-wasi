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

    /// Reads one of zint's fixed size C string fields back, stopping at the
    /// terminator or at the end of the field, whichever comes first.
    ///
    /// The bound is the point: `CStr::from_ptr` walks until it finds a NUL, so
    /// a field zint filled to capacity without terminating would be read past
    /// its end. Bytes that are not UTF-8 are replaced rather than refused,
    /// because a mangled explanation is still better than none.
    ///
    /// Input:  `symbol.fgcolour`, holding `000000ff\0` and then junk
    /// Output: `"000000ff"`
    pub fn read_cstr(field: &[::std::os::raw::c_char]) -> String {
        let bytes = unsafe {
            // Safety: `field` is a live slice of the symbol, and `c_char` has
            // the layout of `u8` whichever sign the target gives it.
            ::std::slice::from_raw_parts(field.as_ptr().cast::<u8>(), field.len())
        };
        let terminated = ::std::ffi::CStr::from_bytes_until_nul(bytes)
            .map(::std::ffi::CStr::to_bytes)
            .unwrap_or(bytes);

        String::from_utf8_lossy(terminated).into_owned()
    }

    #[cfg(test)]
    mod tests {
        use super::{copy_into_cstr, read_cstr};
        use crate::error::Error;
        use std::os::raw::c_char;

        fn field(bytes: &[u8]) -> Vec<c_char> {
            bytes.iter().map(|byte| *byte as c_char).collect()
        }

        /// What zint leaves behind is a terminator followed by whatever was in
        /// the field before, so the tail is not part of the value.
        #[test]
        fn reads_up_to_the_terminator() {
            assert_eq!(read_cstr(&field(b"000000ff\0junk")), "000000ff");
            assert_eq!(read_cstr(&field(b"\0anything")), "");
        }

        /// `CStr::from_ptr` would run off the end of a field like this one.
        /// Reading the whole field is a rendering of what is actually there,
        /// which beats reading memory that is not.
        #[test]
        fn an_unterminated_field_stops_at_its_end() {
            assert_eq!(read_cstr(&field(b"no terminator")), "no terminator");
            assert_eq!(read_cstr(&[]), "");
        }

        /// Zint writes ASCII, but a field it never wrote holds whatever was
        /// there, and a mangled explanation still beats no explanation.
        #[test]
        fn bytes_that_are_not_text_are_replaced_rather_than_refused() {
            assert_eq!(read_cstr(&field(b"ab\xFFc\0")), "ab\u{FFFD}c");
        }

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
