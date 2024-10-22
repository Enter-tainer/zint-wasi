use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Clone)]
pub struct ArgumentList(Vec<OsString>);
#[allow(dead_code)]
impl ArgumentList {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn os_str_args(&self) -> impl Iterator<Item = &OsStr> {
        self.0.iter().map(|it| it.as_os_str())
    }

    /// Chain `other` argument list after this one.
    #[inline]
    pub fn chain(self, other: impl ArgList) -> Self {
        Self(self.0.into_iter().chain(other.os_string_args()).collect())
    }

    pub fn has(&self, key: impl AsRef<OsStr>) -> bool {
        let key = prefix_key(key);
        self.0.iter().any(|it| {
            it.to_string_lossy()
                .split("=")
                .next()
                .map(|it| it == key)
                .unwrap_or_default()
        })
    }

    fn index_kv(&self, key: impl AsRef<OsStr>) -> Option<(usize, Result<&OsStr, usize>)> {
        let key = prefix_key(key);
        self.0.iter().enumerate().find_map(|(i, it)| {
            let (arg, value) = it.as_encoded_bytes().split_at(
                it.as_encoded_bytes()
                    .iter()
                    .enumerate()
                    .find_map(|(i, it)| if *it == b'=' { Some(i) } else { None })
                    .unwrap_or(it.len()),
            );
            unsafe {
                // SAFETY: arg was created with as_encoded_bytes.
                if OsStr::from_encoded_bytes_unchecked(arg) != key {
                    return None;
                };
            }
            let value = if !value.is_empty() {
                unsafe {
                    // SAFETY: value was created with as_encoded_bytes.
                    Ok(OsStr::from_encoded_bytes_unchecked(value))
                }
            } else {
                Err(i + 1)
            };
            Some((i, value))
        })
    }

    #[inline]
    pub fn index(&self, key: impl AsRef<OsStr>) -> Option<usize> {
        self.index_kv(key).map(|it| it.0)
    }

    pub fn get(&self, key: impl AsRef<OsStr>) -> Option<&OsStr> {
        let (_, value) = self.index_kv(key)?;

        let next = match value {
            Ok(value) => return Some(value),
            Err(next) => next,
        };
        let next = self.0.get(next)?;
        if next.len() == 2 && next.as_encoded_bytes().starts_with(b"-") {
            return None;
        }
        if next.len() > 2 && next.as_encoded_bytes().starts_with(b"--") {
            return None;
        }
        Some(next.as_os_str())
    }
}

impl IntoIterator for ArgumentList {
    type Item = OsString;
    type IntoIter = std::vec::IntoIter<OsString>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn prefix_key(key: impl AsRef<OsStr>) -> OsString {
    if key.as_ref().len() == 1 {
        let mut result = OsString::from_str("-").unwrap();
        result.push(key.as_ref());
        result
    } else {
        let mut result = OsString::from_str("--").unwrap();
        result.push(key.as_ref());
        result
    }
}

// The rest are just macros to allow easily converting any slice or tuple of
// AsRef<OsStr> things into ArgumentList.

// Compiling this will be comparatively slower than most other code, but it's
// worth the convenience it provides when passing arguments internally because
// they're almost never of uniform type or in a uniform container.

macro_rules! arg_list_from_slice_or_single {
    ($(($base: ty, |$conv_name: ident| $conv: expr) $(use<$($timed: lifetime),+>)?),* $(,)?) => {
        $(
            impl<$($($timed,)*)?> From<$base> for ArgumentList {
                fn from(arg: $base) -> Self {
                    Self(vec![(|$conv_name: $base| $conv)(arg)])
                }
            }
            impl<$($($timed,)*)?> From<&$base> for ArgumentList {
                fn from(arg: &$base) -> Self {
                    Self(vec![(|$conv_name: &$base| $conv)(arg)])
                }
            }
            impl<$($($timed,)*)? const N: usize> From<[$base; N]> for ArgumentList {
                fn from(args: [$base; N]) -> Self {
                    Self(args.into_iter().map(|$conv_name| $conv).collect())
                }
            }
            impl<$($($timed,)*)? const N: usize> From<&[$base; N]> for ArgumentList {
                fn from(args: &[$base; N]) -> Self {
                    Self(args.into_iter().map(|$conv_name| $conv).collect())
                }
            }
            impl<$($($timed,)*)?> From<Vec<$base>> for ArgumentList {
                fn from(args: Vec<$base>) -> Self {
                    Self(args.into_iter().map(|$conv_name| $conv).collect())
                }
            }
        )*
    };
}

// all compile time and runtime sized slices of following types can be turned
// into an ArgumentList, as well as individual values
arg_list_from_slice_or_single![
    (&'a str,   |it| OsString::from(it)            ) use<'a>,
    (String,    |it| OsString::from(it)            ),
    (&'a OsStr, |it| it.to_os_string()             ) use<'a>,
    (OsString,  |it| it.to_os_string()             ),
    (&'a Path,  |it| it.as_os_str().to_os_string() ) use<'a>,
    (PathBuf,   |it| it.as_os_str().to_os_string() ),
];

impl From<()> for ArgumentList {
    fn from(_: ()) -> Self {
        Self(Vec::new())
    }
}

// any tuple of items that are ArgList is itself an ArgumentList when those are
// chained
macro_rules! arg_list_from_tuple {
    ($(($($i: ident: $S: ident),+)),+ $(,)?) => {
        $(
            impl<$($S),+> From<($($S,)+)> for ArgumentList where
            $($S : ArgList),*
            {
                fn from(($($i,)*): ($($S,)*)) -> Self {
                    Self::new()$(.chain($i))*
                }
            }
        )+
    };
}
macro_rules! or_fewer {
    ($apply: ident ! [$k: ident: $v: ident $(,)?]) => {
        $apply ! (($k: $v));
    };
    ($apply: ident ! [$k_head: ident: $v_head: ident, $($k_tail: ident: $v_tail: ident),+ $(,)?]) => {
        $apply ! [($k_head: $v_head, $($k_tail: $v_tail),*)];
        or_fewer!($apply ! [$($k_tail: $v_tail),*]);
    };
}
// impl arg_list_from_tuple for all tuples of length 1-16
or_fewer!(arg_list_from_tuple![
     s0: S0,   s1: S1,   s2: S2,   s3: S3,
     s4: S4,   s5: S5,   s6: S6,   s7: S7,
     s8: S8,   s9: S9,  s10: S10, s11: S11,
    s12: S12, s13: S13, s14: S14, s15: S15,
]);

pub trait ArgList: Into<ArgumentList> {
    fn into_args(self) -> ArgumentList {
        self.into()
    }
    fn os_string_args(self) -> impl Iterator<Item = OsString> {
        self.into_args().into_iter()
    }
}
impl<T: Into<ArgumentList>> ArgList for T {}

pub trait ChainDirectly {
    fn chain_args(self, other: impl ArgList) -> ArgumentList;
}
impl<T: ArgList> ChainDirectly for T {
    #[inline]
    fn chain_args(self, other: impl ArgList) -> ArgumentList {
        self.into_args().chain(other)
    }
}
