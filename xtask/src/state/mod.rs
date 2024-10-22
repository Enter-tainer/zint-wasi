#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{self, BufRead as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{RwLock, TryLockError};

use crate::log::*;

mod configure;
pub use configure::*;

/// Additional symbol besides state values to resolve
#[rustfmt::skip]
static SYMBOL_MAP: &[(&str, &str)] = &[
    ("$<root>", env!("XTASK_PROJECT_ROOT"))
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

mod global;
pub use global::*;

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

mod refs;
use refs::*;
