//! `openspace-fs` — file system feature crate.
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — file/tree value types.
//! * [`application`] — watcher orchestration.
//! * [`infrastructure`] — OS adapters (notify/fsevents).
//! * [`presenter`] — project tree + file picker rendering.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;
