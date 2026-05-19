use crate::error::SecretError;
use crate::resolver::SecretResolver;

/// A secret resolver backed by the macOS Keychain (or the platform-native
/// credential store on other operating systems).
///
/// On macOS this uses the `keyring` crate to access the Keychain Services.
/// On non-macOS platforms the resolver is a no-op placeholder for v1.
#[derive(Debug, Clone)]
pub struct KeychainResolver {
    service: String,
}

impl KeychainResolver {
    /// Create a new `KeychainResolver` with the given service name.
    ///
    /// On macOS this initializes the platform-native credential store.
    pub fn new(service: impl Into<String>) -> Self {
        #[cfg(target_os = "macos")]
        {
            let _ = keyring::use_native_store(true);
        }
        Self {
            service: service.into(),
        }
    }
}

impl Default for KeychainResolver {
    fn default() -> Self {
        Self::new("openspace")
    }
}

impl SecretResolver for KeychainResolver {
    #[cfg(target_os = "macos")]
    fn resolve(&self, key: &str) -> Result<String, SecretError> {
        let entry = keyring_core::Entry::new(&self.service, key)
            .map_err(|e| SecretError::KeychainAccess(e.to_string()))?;
        entry
            .get_password()
            .map_err(|e| SecretError::KeychainAccess(e.to_string()))
    }

    #[cfg(not(target_os = "macos"))]
    fn resolve(&self, _key: &str) -> Result<String, SecretError> {
        Err(SecretError::KeychainAccess(
            "KeychainResolver is a no-op on non-macOS platforms".to_string(),
        ))
    }
}
