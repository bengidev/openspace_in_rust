//! FS feature application layer.
//!
//! File system watcher orchestration and project-tree refresh
//! orchestration. Watcher plumbing lives here so the
//! infrastructure layer can supply concrete OS adapters without
//! the rest of the app caring about platform specifics.

pub mod fs_watcher;
