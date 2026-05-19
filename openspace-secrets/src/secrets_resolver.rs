use crate::error::SecretError;

/// A trait for resolving secret values by key.
pub trait SecretResolver {
    /// Resolve a secret value for the given key.
    fn resolve(&self, key: &str) -> Result<String, SecretError>;
}
