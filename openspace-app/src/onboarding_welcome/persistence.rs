//! First-run persistence for the welcome window.
//!
//! The welcome window is shown exactly once, on the first launch of
//! the app. After the user dismisses it (either by pressing the
//! primary CTA or the skip action) we drop a sentinel file into the
//! app data directory, and on subsequent launches we route straight
//! to the home shell.
//!
//! Design notes:
//! * The trait is intentionally tiny so we can swap it for an
//!   in-memory fake from tests without pulling in tokio runtimes.
//! * Filesystem operations are blocking but they touch a single
//!   ~0-byte file in the user data dir, so the cost is negligible
//!   and we run them on the main thread during startup. If that ever
//!   becomes a problem we can hop onto a blocking task — the trait
//!   shape supports that without callers caring.
//! * We deliberately do not collapse this into the SQLite store in
//!   `openspace-storage`. A flag-file is enough, has zero schema
//!   weight, and lets the welcome flow remain operational even when
//!   the database is not yet initialised on first launch.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use directories::ProjectDirs;

const SENTINEL_FILENAME: &str = "welcome-completed.flag";

/// Persists whether the welcome window has been completed.
///
/// Implementations must be cheap to clone and cheap to call from the
/// UI thread.
pub trait WelcomePersistence: Send + Sync {
    /// Returns `true` if the welcome window has already been
    /// completed by the user on this device.
    fn is_completed(&self) -> bool;

    /// Marks the welcome window as completed. Idempotent.
    fn mark_completed(&self) -> Result<(), WelcomePersistenceError>;

    /// Clears the completion flag so the welcome window appears on
    /// the next launch. Used by debug builds to ship a "back to
    /// welcome" affordance; not exposed in release builds. Default
    /// implementation is a no-op so out-of-tree implementations do
    /// not break.
    fn reset(&self) -> Result<(), WelcomePersistenceError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WelcomePersistenceError {
    #[error("could not resolve a project data directory")]
    NoProjectDirs,

    #[error("io error while persisting welcome state: {0}")]
    Io(#[from] io::Error),
}

// ---------------------------------------------------------------------------
// Filesystem-backed persistence (production)
// ---------------------------------------------------------------------------

/// Default persistence implementation backed by a sentinel file in the
/// user data directory.
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
    /// directory does not need to exist yet — it is created lazily on
    /// the first `mark_completed` call.
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
        // this idempotent: re-running is a no-op rather than an error.
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

// ---------------------------------------------------------------------------
// In-memory persistence (tests)
// ---------------------------------------------------------------------------

/// In-memory persistence used by tests and previews.
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
