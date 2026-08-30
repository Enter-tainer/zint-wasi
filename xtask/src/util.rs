use std::ffi::{OsStr, OsString};

/// Does the same as [str::replace], only for `byte`s intead of `char`s.
pub trait SliceReplace<Borrowed: ?Sized = Self, Owned = <Self as std::borrow::ToOwned>::Owned> {
    fn replace_slices(&self, from: &Borrowed, to: &Borrowed) -> Owned;
}
impl SliceReplace for [u8] {
    fn replace_slices(&self, from: &[u8], to: &[u8]) -> Vec<u8> {
        if self.len() < from.len() {
            return self.to_vec();
        }
        let mut result = Vec::with_capacity(if from.len() <= to.len() {
            self.len()
        } else {
            (self.len() / from.len() * to.len()) // assume we'll replace everything
                .min(self.len() * 2) // but don't over-allocate
        });

        let mut pos = 0;
        while pos < self.len() {
            if pos + from.len() > self.len() {
                result.extend_from_slice(&self[pos..]);
                break;
            }

            if &self[pos..pos + from.len()] == from {
                result.extend_from_slice(to);
                pos += from.len();
            } else {
                result.push(self[pos]);
                pos += 1;
            }
        }
        result
    }
}
impl SliceReplace<[u8]> for Vec<u8> {
    #[inline(always)]
    fn replace_slices(&self, from: &[u8], to: &[u8]) -> Vec<u8> {
        self.as_slice().replace_slices(from, to)
    }
}
impl SliceReplace for OsStr {
    fn replace_slices(&self, from: &OsStr, to: &OsStr) -> OsString {
        unsafe {
            // SAFETY: All inputs were encoded
            OsString::from_encoded_bytes_unchecked(
                self.as_encoded_bytes()
                    .replace_slices(from.as_encoded_bytes(), to.as_encoded_bytes()),
            )
        }
    }
}
impl SliceReplace<OsStr, OsString> for &OsStr {
    #[inline(always)]
    fn replace_slices(&self, from: &OsStr, to: &OsStr) -> OsString {
        (*self).replace_slices(from, to)
    }
}
impl SliceReplace<OsStr> for OsString {
    #[inline(always)]
    fn replace_slices(&self, from: &OsStr, to: &OsStr) -> OsString {
        self.as_os_str().replace_slices(from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_replace_works() {
        let input = vec![1, 2, 3, 4, 5];
        let output = input.replace_slices(&[2, 3], &[6, 7, 8]);
        assert_eq!(output, &[1, 6, 7, 8, 4, 5])
    }

    #[test]
    fn every_occurrence_is_replaced() {
        let input = b"a-b-c".to_vec();
        assert_eq!(input.replace_slices(b"-", b"::"), b"a::b::c");
    }

    #[test]
    fn a_pattern_that_is_not_there_leaves_the_input_alone() {
        let input = vec![1, 2, 3];
        assert_eq!(input.replace_slices(&[9], &[0]), &[1, 2, 3]);
    }

    /// The path being replaced is longer than most values it is looked for in.
    #[test]
    fn a_pattern_longer_than_the_input_leaves_it_alone() {
        let input = vec![1, 2];
        assert_eq!(input.replace_slices(&[1, 2, 3], &[0]), &[1, 2]);
    }

    #[test]
    fn a_replacement_may_be_shorter_than_what_it_replaces() {
        let input = b"/home/user/project/typst-package".to_vec();
        assert_eq!(
            input.replace_slices(b"/home/user/project", b"$<root>"),
            b"$<root>/typst-package"
        );
    }

    #[test]
    fn a_match_at_the_very_end_is_replaced() {
        let input = vec![1, 2, 3];
        assert_eq!(input.replace_slices(&[2, 3], &[4]), &[1, 4]);
    }

    /// Overlapping matches are taken left to right, so the second `aa` here
    /// starts after the first one was consumed.
    #[test]
    fn overlapping_matches_are_taken_left_to_right() {
        let input = b"aaaa".to_vec();
        assert_eq!(input.replace_slices(b"aa", b"b"), b"bb");

        let input = b"aaa".to_vec();
        assert_eq!(input.replace_slices(b"aa", b"b"), b"ba");
    }

    /// Paths are `OsString`s, which is what the replacement is really used on.
    #[test]
    fn os_strings_are_replaced_the_same_way() {
        let input = OsString::from("/home/user/project/typst-package");

        assert_eq!(
            input.replace_slices(OsStr::new("/home/user/project"), OsStr::new("$<root>")),
            OsString::from("$<root>/typst-package")
        );
    }
}
