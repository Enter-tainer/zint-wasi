use super::*;

pub(super) const STATE_PATH: &str = concat![env!("XTASK_PROJECT_ROOT"), "/xtask/state"];

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
