use super::*;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

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