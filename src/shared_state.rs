use crate::ResourceHandlerError;
use std::sync::{Arc, Mutex};

pub(crate) struct SharedState<T> {
    state: Arc<Mutex<Option<T>>>,
}

impl<T> Clone for SharedState<T> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<T> SharedState<T> {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
        }
    }

    /// Replaces the current shared state with `new_state`.
    /// Returns an error if the lock cannot be acquired.
    pub(crate) fn set(&self, new_state: T) -> Result<(), ResourceHandlerError> {
        let mut guard = self.state.lock()?;
        *guard = Some(new_state);
        Ok(())
    }

    /// Calls `f` with a mutable reference to the current state.
    /// Returns an error if the state is empty or the lock cannot be acquired.
    pub(crate) fn with_mut<R>(
        &self,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, ResourceHandlerError> {
        let mut guard = self.state.lock()?;
        let state = guard
            .as_mut()
            .ok_or(ResourceHandlerError::StateNotInitializedError)?;
        Ok(f(state))
    }

    /// Calls `f` with an reference to the current state.
    /// Returns an error if the state is empty or the lock couldn't be acquired.
    pub(crate) fn with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, ResourceHandlerError> {
        let guard = self.state.lock()?;
        let state = guard
            .as_ref()
            .ok_or(ResourceHandlerError::StateNotInitializedError)?;
        Ok(f(state))
    }
}
