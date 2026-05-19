//! In-memory welcome persistence used by tests and previews.
//!
//! Lets the routing layer be exercised end-to-end without touching
//! the filesystem.

use std::sync::{Arc, Mutex};

use crate::domain::{WelcomePersistence, WelcomePersistenceError};

#[derive(Debug, Clone, Default)]
pub struct InMemoryWelcomePersistence {
    completed: Arc<Mutex<bool>>,
}

impl InMemoryWelcomePersistence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn already_completed() -> Self {
        Self {
            completed: Arc::new(Mutex::new(true)),
        }
    }
}

impl WelcomePersistence for InMemoryWelcomePersistence {
    fn is_completed(&self) -> bool {
        *self.completed.lock().unwrap()
    }

    fn mark_completed(&self) -> Result<(), WelcomePersistenceError> {
        *self.completed.lock().unwrap() = true;
        Ok(())
    }

    fn reset(&self) -> Result<(), WelcomePersistenceError> {
        *self.completed.lock().unwrap() = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_persistence_round_trips() {
        let store = InMemoryWelcomePersistence::new();
        assert!(!store.is_completed());
        store.mark_completed().unwrap();
        assert!(store.is_completed());
    }

    #[test]
    fn in_memory_already_completed_starts_true() {
        let store = InMemoryWelcomePersistence::already_completed();
        assert!(store.is_completed());
    }

    #[test]
    fn in_memory_reset_clears_completion() {
        let store = InMemoryWelcomePersistence::already_completed();
        store.reset().unwrap();
        assert!(!store.is_completed());
    }
}
