#![allow(dead_code)]

use crate::log::*;
use std::io::{self, BufRead as _, Write as _};
use std::path::{Path, PathBuf};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{RwLock, TryLockError},
};

pub(in super) const STATE_PATH: &str = "./xtask/state";

#[derive(Default)]
pub struct State {
    source: Option<PathBuf>,
    items: BTreeMap<String, String>,
    environment: HashMap<String, String>,
}
impl State {
    fn global() -> &'static RwLock<State> {
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
    pub fn global_read() -> StateRead {
        match Self::global().try_read() {
            Ok(read) => StateRead { read },
            Err(TryLockError::WouldBlock) => panic!("state mutably borrowed"),
            Err(TryLockError::Poisoned(_)) => panic!("state poisoned"),
        }
    }
    pub fn global_write() -> StateWrite {
        match Self::global().try_write() {
            Ok(write) => StateWrite { write },
            Err(TryLockError::WouldBlock) => panic!("state already borrowed"),
            Err(TryLockError::Poisoned(_)) => panic!("state poisoned"),
        }
    }

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
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                })
            })
            .collect();

        let items = items?;
        let mut environment = HashMap::new();
        for (key, val) in std::env::vars() {
            if let Some(key) = key.strip_prefix("XTASK_") {
                environment.insert(key.to_string(), val);
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
            output.write_all(v.as_bytes())?;
            output.write_all(b"\n")?;
        }
        output.flush()
    }
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        self.items
            .get(key.as_ref())
            .map(|it| it.as_str())
            .or_else(|| self.environment.get(key.as_ref()).map(|it| it.as_str()))
    }
    pub fn set(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) {
        self.items
            .insert(key.as_ref().to_string(), value.as_ref().to_string());
    }
}

impl<S> std::ops::Index<S> for State
where
    S: AsRef<str>,
{
    type Output = str;

    fn index(&self, index: S) -> &Self::Output {
        self.items
            .get(index.as_ref())
            .expect("state is missing expected entry")
    }
}

#[macro_export]
macro_rules! state {
    ($key: tt) => {
        $crate::state::State::global_read()
            .get(stringify!($key))
            .expect(concat!["state is missing '", stringify!($key), "' key"])
    };
    ($key: tt, default: $default: literal) => {
        $crate::state::State::global_read()
            .get(stringify!($key))
            .unwrap_or($default)
    };
    ($key: tt, default: || $else: block) => {
        $crate::state::State::global_read()
            .get(stringify!($key))
            .map(|it| it.to_string())
            .unwrap_or_else(|| $else)
    };
}
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

mod _impl {
    use super::*;
    use std::sync::{RwLockReadGuard, RwLockWriteGuard};

    pub struct StateRead {
        pub(super) read: RwLockReadGuard<'static, State>,
    }
    impl std::ops::Deref for StateRead {
        type Target = State;

        fn deref(&self) -> &Self::Target {
            &self.read
        }
    }

    pub struct StateWrite {
        pub(super) write: RwLockWriteGuard<'static, State>,
    }
    impl std::ops::Deref for StateWrite {
        type Target = State;

        fn deref(&self) -> &Self::Target {
            &self.write
        }
    }
    impl std::ops::DerefMut for StateWrite {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.write
        }
    }
}
use _impl::*;
