use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SecretError {
    #[error("Secret not found for key: {0}")]
    NotFound(String),
    #[error("Keychain access failed: {0}")]
    KeychainAccess(String),
    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),
    #[error("Invalid secret key: {0}")]
    InvalidKey(String),
}
