use super::*;
use std::ffi::{OsStr, OsString};

/// Allows replacing all keys from [`SYMBOL_MAP`] and (if provided) [`State`]
/// with associated values.
pub trait Configure {
    type Result;
    fn configure<'s>(self, state: impl OptionalState<'s>) -> Self::Result;
    fn unconfigure<'s>(self, state: impl OptionalState<'s>) -> Self::Result;
}

/// Convenience macro for calling [`Configure`].
#[macro_export]
macro_rules! configure {
    // cheapest variant that doesn't resolve state variables
    ($what: expr) => {
        $crate::state::Configure::configure($what, ())
    };
    // resolves state variables using global state
    ($what: expr, state: GlobalState) => {
        $crate::state::Configure::configure($what, $crate::state::GlobalState)
    };
    // resolves state variables using specified state
    ($what: expr, state: $state: expr) => {
        $crate::state::Configure::configure($what, $state)
    };
}
#[allow(unused_imports)]
pub use crate::configure;

/// Much like [`AsRef`], but returns [`Option<StateRead<'s>>`] for different
/// types.
///
/// Consuming function decides whether to fallback to global state or not to use
/// state at all.
pub trait OptionalState<'s> {
    fn as_option(&self) -> Option<StateRead<'s>>;
}
impl<'s> OptionalState<'s> for () {
    fn as_option(&self) -> Option<StateRead<'s>> {
        None
    }
}
impl<'s> OptionalState<'s> for &State {
    fn as_option(&self) -> Option<StateRead<'s>> {
        None
    }
}
impl<'s> OptionalState<'s> for GlobalState {
    fn as_option(&self) -> Option<StateRead<'s>> {
        Some(global_read())
    }
}

/// Much like [`AsMut`], but returns [`Option<StateWrite<'s>>`] for different
/// types.
///
/// Consuming function decides whether to fallback to global state or not to use
/// state at all.
pub trait OptionalStateMut<'s> {
    fn into_option_mut(self) -> Option<StateWrite<'s>>;
}
impl OptionalStateMut<'static> for () {
    fn into_option_mut(self) -> Option<StateWrite<'static>> {
        None
    }
}
impl<'s> OptionalStateMut<'s> for StateWrite<'s> {
    fn into_option_mut(self) -> Option<StateWrite<'s>> {
        Some(self)
    }
}
impl OptionalStateMut<'static> for GlobalState {
    fn into_option_mut(self) -> Option<StateWrite<'static>> {
        Some(GlobalState::as_mut())
    }
}

impl<I: _coalesce::Input> Configure for I {
    type Result = I::Output;

    fn configure<'s>(self, state: impl OptionalState<'s>) -> Self::Result {
        let mut it = self.input();
        for (from, to) in SYMBOL_MAP {
            it = it.replace(from, to);
        }
        if let Some(state) = state.as_option() {
            for (from, to) in state.iter() {
                let from = format!("$<{}>", from);
                it = it.replace(from, to);
            }
        }
        I::output(it)
    }
    fn unconfigure<'s>(self, state: impl OptionalState<'s>) -> Self::Result {
        let mut it = self.input();
        if let Some(state) = state.as_option() {
            for (from, to) in state.iter() {
                let from = format!("$<{}>", from);
                it = it.replace(from, to);
            }
        }
        for (from, to) in SYMBOL_MAP {
            it = it.replace(from, to);
        }
        I::output(it)
    }
}

trait VecReplace {
    fn replace<F, T>(&self, from: F, to: T) -> Self
    where
        F: AsRef<[u8]>,
        T: AsRef<[u8]>;
}
impl VecReplace for Vec<u8> {
    /// Mostly inlined from bstr::ByteVec::replace.
    fn replace<F, T>(&self, from: F, to: T) -> Self
    where
        F: AsRef<[u8]>,
        T: AsRef<[u8]>,
    {
        let (from, to) = (from.as_ref(), to.as_ref());
        let mut dest = if to.len() < from.len() {
            Vec::with_capacity(self.len())
        } else {
            Vec::with_capacity((self.len() / to.len() * from.len()).min(self.len() * 2))
        };

        let mut last = 0;
        for start in memchr::memmem::find_iter(self, from) {
            dest.extend_from_slice(&self.as_slice()[last..start]);
            dest.extend_from_slice(to);
            last = start + from.len();
        }
        dest.extend_from_slice(&self.as_slice()[last..]);
        dest
    }
}

/// Adapters for various types so they can be used with [`Configure`] trait.
mod _coalesce {
    use super::*;

    pub trait Input {
        type Output;
        fn input(self) -> Vec<u8>;
        fn output(value: Vec<u8>) -> Self::Output;
    }
    impl Input for &OsStr {
        type Output = OsString;
        fn input(self) -> Vec<u8> {
            Input::input(self.to_owned())
        }
        fn output(value: Vec<u8>) -> Self::Output {
            <OsString as Input>::output(value)
        }
    }
    impl Input for OsString {
        type Output = OsString;

        #[cfg(unix)]
        fn input(self) -> Vec<u8> {
            use std::os::unix::ffi::OsStringExt as _;
            self.into_vec()
        }
        #[cfg(unix)]
        fn output(value: Vec<u8>) -> Self::Output {
            use std::os::unix::ffi::OsStringExt as _;
            OsString::from_vec(value)
        }

        #[cfg(windows)]
        fn input(self) -> Vec<u8> {
            use std::os::windows::ffi::OsStringExt as _;

            // Converts OsString (UTF-16 encoded) to a Vec<u8> on Windows
            let utf16_vec: Vec<u16> = self.encode_wide().collect();
            // Convert the UTF-16 vector to a byte vector
            utf16_vec.iter().flat_map(|&x| x.to_le_bytes()).collect()
        }
        #[cfg(windows)]
        fn output(value: Vec<u8>) -> Self::Output {
            use std::os::windows::ffi::OsStringExt as _;

            let utf16_vec: Vec<u16> = value
                .chunks(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
            OsString::from_wide(&utf16_vec)
        }
    }
    impl Input for &Path {
        type Output = PathBuf;
        fn input(self) -> Vec<u8> {
            Input::input(self.as_os_str().to_owned())
        }
        fn output(value: Vec<u8>) -> Self::Output {
            PathBuf::from(<OsString as Input>::output(value))
        }
    }
    impl Input for PathBuf {
        type Output = PathBuf;
        fn input(self) -> Vec<u8> {
            Input::input(self.into_os_string())
        }
        fn output(value: Vec<u8>) -> Self::Output {
            PathBuf::from(<OsString as Input>::output(value))
        }
    }
    impl Input for &PathBuf {
        type Output = PathBuf;
        fn input(self) -> Vec<u8> {
            Input::input(self.as_os_str().to_owned())
        }
        fn output(value: Vec<u8>) -> Self::Output {
            PathBuf::from(<OsString as Input>::output(value))
        }
    }
    impl Input for &str {
        type Output = String;
        fn input(self) -> Vec<u8> {
            self.into()
        }
        fn output(value: Vec<u8>) -> Self::Output {
            unsafe {
                // SAFETY: Configuration assumes UTF8 encoding
                String::from_utf8_unchecked(value)
            }
        }
    }
    impl Input for String {
        type Output = String;
        fn input(self) -> Vec<u8> {
            self.into()
        }
        fn output(value: Vec<u8>) -> Self::Output {
            unsafe {
                // SAFETY: Configuration assumes UTF8 encoding
                String::from_utf8_unchecked(value)
            }
        }
    }
    impl Input for &String {
        type Output = String;
        fn input(self) -> Vec<u8> {
            self.as_bytes().to_owned()
        }
        fn output(value: Vec<u8>) -> Self::Output {
            unsafe {
                // SAFETY: Configuration assumes UTF8 encoding
                String::from_utf8_unchecked(value)
            }
        }
    }
}
