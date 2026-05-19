//! `openspace-editor` — editor mode feature crate.
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — pure document/file value types.
//! * [`application`] — buffer model + command catalogue.
//! * [`infrastructure`] — file I/O + parser adapters.
//! * [`presenter`] — Iced-side rendering for the editor
//!   workspace surface.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;

pub use application::EditorCommands;
