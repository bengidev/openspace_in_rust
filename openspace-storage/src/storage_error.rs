use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid mode: {0}")]
    InvalidMode(String),
    #[error("Invalid permission profile: {0}")]
    InvalidPermissionProfile(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not determine project directories")]
    NoProjectDirs,
}
