//! Persistence contract for the welcome flag.
//!
//! The welcome window is shown exactly once, on the first launch of
//! the app. After the user dismisses it (either by pressing the
//! primary CTA or the skip action) the implementation marks a
//! sentinel; on subsequent launches the router queries this trait
//! and routes straight to the home shell.
//!
//! The trait is intentionally tiny so it can be swapped for an
//! in-memory fake from tests without pulling in tokio runtimes or a
//! real filesystem.

use std::io;

/// Persists whether the welcome window has been completed.
///
/// Implementations must be cheap to clone behind an `Arc` and cheap
/// to call from the UI thread.
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
