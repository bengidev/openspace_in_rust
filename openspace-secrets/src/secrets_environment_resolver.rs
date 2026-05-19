use crate::error::SecretError;
use crate::resolver::SecretResolver;

/// A placeholder resolver that reads secrets from environment variables.
///
/// Full implementation will be added in a future slice.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentResolver;

impl EnvironmentResolver {
    /// Create a new `EnvironmentResolver`.
    pub fn new() -> Self {
        Self
    }
}

impl SecretResolver for EnvironmentResolver {
    fn resolve(&self, key: &str) -> Result<String, SecretError> {
        std::env::var(key).map_err(|_| SecretError::EnvVarNotSet(key.to_string()))
    }
}
