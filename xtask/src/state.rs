#![allow(dead_code)]

use crate::{log::*, util::SliceReplace};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ffi::{OsStr, OsString},
    io::{self, BufRead as _, Write as _},
    path::{Path, PathBuf},
    sync::{RwLock, TryLockError},
};

/// Additional symbol besides state values to resolve
#[rustfmt::skip]
static SYMBOL_MAP: &[(&str, &str)] = &[
    ("$<root>", env!("PROJECT_ROOT"))
];

/// A simple key-value store that can be loaded and saved.
#[derive(Default)]
pub struct State {
    source: Option<PathBuf>,
    items: BTreeMap<String, String>,
    environment: HashMap<String, String>,
}
impl State {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let input = std::fs::File::open(path.as_ref()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("unable to find state file: {}", path.as_ref().display()),
            )
        })?;
        let input = io::BufReader::new(input);

        let items: Result<BTreeMap<String, String>, _> = input
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                let Ok(line) = line else {
                    return Some(line.map(|it| (i, it)));
                };
                let line = line.trim();
                if line.starts_with('#') {
                    return None;
                }

                Some(Ok((
                    i,
                    line.split_once('#')
                        .map(|(expr, _)| expr.trim())
                        .unwrap_or(line)
                        .to_string(),
                )))
            })
            .map(|it| {
                it.and_then(|(i, line)| {
                    line.split_once("=")
                        .ok_or(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "{}:{i}: state variable doesn't contain '=' separator",
                                path.as_ref().display()
                            ),
                        ))
                        .map(|(k, v)| (k.to_string(), v.configure(())))
                })
            })
            .collect();

        let items = items?;
        let mut environment = HashMap::new();
        for (key, val) in std::env::vars() {
            if let Some(key) = key.strip_prefix("XTASK_") {
                environment.insert(key.to_string(), val.configure(()));
            }
        }

        Ok(Self {
            source: Some(path.as_ref().to_path_buf()),
            items,
            environment,
        })
    }
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let output = std::fs::File::create(path.as_ref())?;
        let mut output = io::BufWriter::new(output);
        for (k, v) in &self.items {
            output.write_all(k.as_bytes())?;
            output.write_all(b"=")?;
            output.write_all(v.unconfigure(()).as_bytes())?;
            output.write_all(b"\n")?;
        }
        output.flush()
    }
    /// Get `key` entry value.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        self.environment
            .get(key.as_ref())
            .map(|it| it.as_str())
            .or_else(|| self.items.get(key.as_ref()).map(|it| it.as_str()))
    }
    /// Set `key` entry to `value`.
    pub fn set(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) {
        self.items
            .insert(key.as_ref().to_string(), value.as_ref().to_string());
    }
    /// Set `key` entry to `value` for current run only.
    pub fn set_temporary(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) {
        self.environment
            .insert(key.as_ref().to_string(), value.as_ref().to_string());
    }
    /// Iterate over entries.
    pub fn iter(&self) -> impl Iterator<Item = (String, String)> + '_ {
        let mut found = HashSet::new();
        self.environment
            .iter()
            .chain(self.items.iter())
            .filter_map(move |(k, v)| {
                if found.contains(k) {
                    None
                } else {
                    found.insert(k.clone());
                    Some((k.clone(), v.clone()))
                }
            })
    }
}

/// Accessor for global state.
///
/// Also works as state value for functions that accept [`OptionalState`] or
/// [`OptionalStateMut`].
pub struct GlobalState;

impl GlobalState {
    pub fn as_ref() -> StateRead<'static> {
        global_read()
    }
    pub fn as_mut() -> StateWrite<'static> {
        global_write()
    }
    pub fn get(key: impl AsRef<str>) -> Option<String> {
        Self::as_ref().get(key).map(|it| it.to_string())
    }
    pub fn set(key: impl AsRef<str>, value: impl AsRef<str>) {
        Self::as_mut().set(key, value)
    }
    pub fn set_temporary(key: impl AsRef<str>, value: impl AsRef<str>) {
        Self::as_mut().set_temporary(key, value)
    }
    pub fn save() -> io::Result<()> {
        global_read().save(STATE_PATH)
    }
}

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
    ($what: expr) => {
        $crate::state::Configure::configure($what, ())
    };
    ($what: expr, state: GlobalState) => {
        $crate::state::Configure::configure($what, $crate::state::GlobalState)
    };
    ($what: expr, state: $state: expr) => {
        $crate::state::Configure::configure($what, $state)
    };
}
#[allow(unused_imports)]
pub use crate::configure;

/// Get value from global state or default
#[macro_export]
macro_rules! state {
    ($key: tt) => {
        $crate::state::GlobalState::get(stringify!($key))
            .expect(concat!["state is missing '", stringify!($key), "' key"])
    };
    ($key: tt, default: $default: literal $(, state: $state: expr)?) => {
        $crate::state::GlobalState::get(stringify!($key))
            .unwrap_or_else(|| $crate::configure!($default $(, state: $state)?))
    };
    ($key: tt, default: || $else: block $(, state: $state: expr)?) => {
        $crate::state::GlobalState::get(stringify!($key))
            .unwrap_or_else(|| $crate::configure!($else $(, state: $state)?))
    };
}

/// Get path from global state or default
#[macro_export]
macro_rules! state_path {
    ($key: tt) => {
        std::path::PathBuf::from($crate::state!($key))
    };
    ($key: tt, default: $default: literal) => {
        std::path::PathBuf::from($crate::state!($key, default: $default))
    };
    ($key: tt, default: || $else: block) => {
        std::path::PathBuf::from($crate::state!($key, default: || $else))
    };
}

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

// SECTION: Various Configure implementations

macro_rules! impl_cfg_base {
    ($(|$it: ident: $R: ty, $from: ident: $T: ty, $to: ident: $_ignored: ty| $replace: block),* $(,)?) => {
        $(impl Configure for $T {
            type Result = $R;

            fn configure<'s>(self, state: impl OptionalState<'s>) -> Self::Result {
                let mut $it = self.to_owned();
                for ($from, $to) in SYMBOL_MAP {
                    $it = (|$it: $R, $from, $to| $replace)($it, $from, $to);
                }
                if let Some(state) = state.as_option() {
                    for ($from, $to) in state.iter() {
                        let $from = format!("$<{}>", $from);
                        $it = (|$it: $R, $from, $to| $replace)($it, &$from, &$to);
                    }
                }
                $it
            }
            fn unconfigure<'s>(self, state: impl OptionalState<'s>) -> Self::Result {
                let mut $it = self.to_owned();
                if let Some(state) = state.as_option() {
                    for ($from, $to) in state.iter() {
                        let $from = format!("$<{}>", $from);
                        $it = (|$it: $R, $from, $to| $replace)($it, &$to, &$from);
                    }
                }
                for ($from, $to) in SYMBOL_MAP {
                    $it = (|$it: $R, $from, $to| $replace)($it, $to, $from);
                }
                $it
            }
        })*
    };
}
impl_cfg_base![
    |it: String, from: &str, to: &str| { it.replace(from, to) },
    |it: OsString, from: &OsStr, to: &OsStr| {
        it.replace_slices(OsStr::new(from), OsStr::new(to))
    },
];

macro_rules! impl_cfg_indirect_one {
    ($cast: ident, T = $T: ty, R = $R: ty) => {
        impl_cfg_indirect_one!($cast, T = $T, R = $R, |it| { it });
    };
    ($cast: ident, T = $T: ty, R = $R: ty, |$it: ident| $to_result: expr) => {
        impl Configure for $T {
            type Result = $R;
            #[inline(always)]
            fn configure<'s>(self, state: impl OptionalState<'s>) -> Self::Result {
                (|$it| $to_result)(self.$cast().configure(state))
            }
            #[inline(always)]
            fn unconfigure<'s>(self, state: impl OptionalState<'s>) -> Self::Result {
                (|$it| $to_result)(self.$cast().unconfigure(state))
            }
        }
    };
}
macro_rules! impl_cfg_indirect {
    ($({$cast: ident, T = $T: ty, R = $R: ty $(, |$it: ident| $to_result: expr )?}),* $(,)?) => {
        $(
            impl_cfg_indirect_one!($cast, T=$T, R=$R $(,|$it| $to_result)?);
        )*
    };
}
impl_cfg_indirect![
    { as_str,    T = String,    R = String   },
    { as_str,    T = &String,   R = String   },
    { as_os_str, T = OsString,  R = OsString },
    { as_os_str, T = &OsString, R = OsString },
    { as_os_str, T = &Path,     R = PathBuf, |it| PathBuf::from(it) },
    { as_os_str, T = PathBuf,   R = PathBuf, |it| PathBuf::from(it) },
    { as_os_str, T = &PathBuf,  R = PathBuf, |it| PathBuf::from(it) },
];

// !SECTION: Various Configure implementations

mod _impl {
    use super::*;
    use std::sync::{RwLockReadGuard, RwLockWriteGuard};

    pub(super) const STATE_PATH: &str = concat![env!("PROJECT_ROOT"), "/xtask/state"];

    pub(super) fn global_state() -> &'static RwLock<State> {
        use std::sync::{OnceLock, RwLock};

        static STATE: OnceLock<RwLock<State>> = OnceLock::new();
        STATE.get_or_init(|| {
            let value = match State::load(STATE_PATH) {
                Ok(it) => it,
                Err(not_found) if not_found.kind() == io::ErrorKind::NotFound => State::new(),
                Err(other) => {
                    error!(other);
                    std::process::abort()
                }
            };

            RwLock::new(value)
        })
    }
    pub(super) fn global_read() -> StateRead<'static> {
        match global_state().try_read() {
            Ok(read) => StateRead::Guard(StateReadGuard { read }),
            Err(TryLockError::WouldBlock) => panic!("state mutably borrowed"),
            Err(TryLockError::Poisoned(_)) => panic!("state poisoned"),
        }
    }
    pub(super) fn global_write() -> StateWrite<'static> {
        match global_state().try_write() {
            Ok(write) => StateWrite::Guard(StateWriteGuard { write }),
            Err(TryLockError::WouldBlock) => panic!("state already borrowed"),
            Err(TryLockError::Poisoned(_)) => panic!("state poisoned"),
        }
    }

    // SECTION: Accessing state by referefence

    pub enum StateRead<'s> {
        Guard(StateReadGuard<'s>),
        Reference(&'s State),
    }
    pub struct StateReadGuard<'s> {
        pub(super) read: RwLockReadGuard<'s, State>,
    }
    impl<'s> AsRef<State> for StateRead<'s> {
        fn as_ref(&self) -> &State {
            match self {
                StateRead::Guard(guard) => &guard.read,
                StateRead::Reference(it) => it,
            }
        }
    }
    impl<'s> std::ops::Deref for StateRead<'s> {
        type Target = State;
        fn deref(&self) -> &Self::Target {
            self.as_ref()
        }
    }

    pub enum StateWrite<'s> {
        Guard(StateWriteGuard<'s>),
        Reference(&'s mut State),
    }
    pub struct StateWriteGuard<'s> {
        pub(super) write: RwLockWriteGuard<'s, State>,
    }
    impl<'s> AsRef<State> for StateWrite<'s> {
        fn as_ref(&self) -> &State {
            match self {
                StateWrite::Guard(guard) => &guard.write,
                StateWrite::Reference(it) => it,
            }
        }
    }
    impl<'s> AsMut<State> for StateWrite<'s> {
        fn as_mut(&mut self) -> &mut State {
            match self {
                StateWrite::Guard(guard) => &mut guard.write,
                StateWrite::Reference(it) => it,
            }
        }
    }
    impl<'s> std::ops::Deref for StateWrite<'s> {
        type Target = State;

        fn deref(&self) -> &Self::Target {
            self.as_ref()
        }
    }
    impl<'s> std::ops::DerefMut for StateWrite<'s> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.as_mut()
        }
    }

    // !SECTION: Accessing state by referefence
}
use _impl::*;
