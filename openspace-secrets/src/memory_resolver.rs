use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::SecretError;
use crate::resolver::SecretResolver;

/// A fully in-memory secret resolver for testing and ephemeral storage.
#[derive(Debug, Clone, Default)]
pub struct MemoryResolver {
    store: Arc<RwLock<HashMap<String, String>>>,
}

impl MemoryResolver {
    /// Create a new empty `MemoryResolver`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair into the in-memory store.
    pub fn insert(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut store = self.store.write().unwrap();
        store.insert(key.into(), value.into());
    }

    /// Remove a key from the in-memory store.
    pub fn remove(&self, key: &str) -> Option<String> {
        let mut store = self.store.write().unwrap();
        store.remove(key)
    }

    /// Clear all entries from the in-memory store.
    pub fn clear(&self) {
        let mut store = self.store.write().unwrap();
        store.clear();
    }
}

impl SecretResolver for MemoryResolver {
    fn resolve(&self, key: &str) -> Result<String, SecretError> {
        let store = self.store.read().unwrap();
        store
            .get(key)
            .cloned()
            .ok_or_else(|| SecretError::NotFound(key.to_string()))
    }
}
