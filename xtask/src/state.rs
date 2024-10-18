use std::collections::HashMap;
use std::io::{self, BufRead as _, Write as _};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct State {
    source: Option<PathBuf>,
    items: HashMap<String, String>,
    environment: HashMap<String, String>,
}
impl State {
    pub fn global() -> StateAccess {
        use std::sync::{Mutex, OnceLock};

        const STATE_PATH: &str = "./xtask/state";
        static STATE: OnceLock<Mutex<State>> = OnceLock::new();

        let mutex = STATE.get_or_init(|| {
            let value = match State::load(STATE_PATH) {
                Ok(it) => it,
                Err(not_found) if not_found.kind() == io::ErrorKind::NotFound => State::new(),
                Err(other) => {
                    eprintln!("{other}");
                    std::process::abort()
                }
            };

            Mutex::new(value)
        });

        let mutex = match mutex.try_lock() {
            Ok(it) => it,
            Err(std::sync::TryLockError::WouldBlock) => panic!("global state already locked"),
            Err(std::sync::TryLockError::Poisoned(_)) => panic!("global state poisoned"),
        };

        StateAccess { mutex }
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

        let items: Result<HashMap<String, String>, _> = input
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

impl Drop for State {
    fn drop(&mut self) {
        if let Some(source) = &self.source {
            if self.save(source).is_err() {
                eprintln!("unable to update xtask state file: {}", source.display())
            }
        }
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
        $crate::state::State::global()
            .get(stringify!($key))
            .expect(concat!["state is missing '", stringify!($key), "' key"])
    };
    ($key: tt, default: $default: literal) => {
        $crate::state::State::global()
            .get(stringify!($key))
            .unwrap_or($default)
    };
    ($key: literal, default: || $else: block) => {
        $crate::state::State::global()
            .get(stringify!($key))
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
    ($key: literal, default: || $else: block) => {
        std::path::PathBuf::from($crate::state!($key, default: || $else))
    };
}

mod _impl {
    use super::*;
    use std::sync::MutexGuard;

    pub struct StateAccess {
        pub(super) mutex: MutexGuard<'static, State>,
    }
    impl std::ops::Deref for StateAccess {
        type Target = State;

        fn deref(&self) -> &Self::Target {
            &self.mutex
        }
    }
    impl std::ops::DerefMut for StateAccess {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.mutex
        }
    }
}
use _impl::*;
