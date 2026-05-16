use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CoreError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Feature not found: {0}")]
    FeatureNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
}
