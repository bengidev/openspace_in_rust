//! Filesystem-backed welcome persistence.
//!
//! After the user dismisses the welcome window we drop a sentinel
//! file into the app data directory; on subsequent launches the
//! router queries [`WelcomePersistence::is_completed`] and routes
//! straight to the home shell.
//!
//! Filesystem operations are blocking but they touch a single
//! ~0-byte file, so we run them on the main thread during startup.
//! If that ever becomes a problem we can hop onto a blocking task
//! — the trait shape supports that without callers caring.
//!
//! We deliberately do not collapse this into the SQLite store in
//! `openspace-storage`. A flag-file is enough, has zero schema
//! weight, and lets the welcome flow remain operational even when
//! the database is not yet initialised on first launch.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::domain::{WelcomePersistence, WelcomePersistenceError};

const SENTINEL_FILENAME: &str = "welcome-completed.flag";

/// Default persistence implementation backed by a sentinel file in
/// the user data directory.
#[derive(Debug, Clone)]
pub struct FileWelcomePersistence {
    sentinel_path: PathBuf,
}

impl FileWelcomePersistence {
    /// Resolves the canonical path under the platform data directory
    /// (e.g. `~/Library/Application Support/openspace` on macOS).
    pub fn from_project_dirs() -> Result<Self, WelcomePersistenceError> {
        let proj_dirs = ProjectDirs::from("com", "openspace", "openspace")
            .ok_or(WelcomePersistenceError::NoProjectDirs)?;
        Ok(Self::new_at(proj_dirs.data_dir()))
    }

    /// Creates a persistence handle at an explicit directory. The
    /// directory does not need to exist yet — it is created lazily
    /// on the first `mark_completed` call.
    pub fn new_at<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            sentinel_path: dir.as_ref().join(SENTINEL_FILENAME),
        }
    }

    pub fn sentinel_path(&self) -> &Path {
        &self.sentinel_path
    }
}

impl WelcomePersistence for FileWelcomePersistence {
    fn is_completed(&self) -> bool {
        self.sentinel_path.exists()
    }

    fn mark_completed(&self) -> Result<(), WelcomePersistenceError> {
        if let Some(parent) = self.sentinel_path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Touch a zero-byte file. Using `OpenOptions::create` keeps
        // this idempotent: re-running is a no-op rather than an
        // error.
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&self.sentinel_path)?;
        Ok(())
    }

    fn reset(&self) -> Result<(), WelcomePersistenceError> {
        match fs::remove_file(&self.sentinel_path) {
            Ok(()) => Ok(()),
            // Already absent — nothing to clear.
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(WelcomePersistenceError::Io(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn file_persistence_starts_incomplete() {
        let tmp = TempDir::new().unwrap();
        let store = FileWelcomePersistence::new_at(tmp.path());
        assert!(!store.is_completed());
    }

    #[test]
    fn mark_completed_persists_across_handles() {
        let tmp = TempDir::new().unwrap();
        let first = FileWelcomePersistence::new_at(tmp.path());
        first.mark_completed().unwrap();
        assert!(first.is_completed());

        let second = FileWelcomePersistence::new_at(tmp.path());
        assert!(second.is_completed());
    }

    #[test]
    fn mark_completed_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let store = FileWelcomePersistence::new_at(tmp.path());
        store.mark_completed().unwrap();
        store.mark_completed().unwrap();
        assert!(store.is_completed());
    }

    #[test]
    fn mark_completed_creates_missing_parent_directory() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("nested").join("dir");
        let store = FileWelcomePersistence::new_at(&nested);
        assert!(!nested.exists());
        store.mark_completed().unwrap();
        assert!(store.is_completed());
        assert!(nested.exists());
    }

    #[test]
    fn file_reset_removes_sentinel() {
        let tmp = TempDir::new().unwrap();
        let store = FileWelcomePersistence::new_at(tmp.path());
        store.mark_completed().unwrap();
        assert!(store.is_completed());
        store.reset().unwrap();
        assert!(!store.is_completed());
    }

    #[test]
    fn file_reset_when_absent_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let store = FileWelcomePersistence::new_at(tmp.path());
        // No mark_completed beforehand; reset should still succeed.
        store.reset().unwrap();
        assert!(!store.is_completed());
    }
}
