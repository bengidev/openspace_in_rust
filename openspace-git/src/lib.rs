//! `openspace-git` — git context feature crate.
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — diff/commit/branch value types.
//! * [`application`] — repository orchestration.
//! * [`infrastructure`] — concrete git backends.
//! * [`presenter`] — diff review + staging UI.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;
