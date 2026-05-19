//! `openspace-terminal` — terminal mode feature crate.
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — pure value types describing terminal
//!   state.
//! * [`application`] — use cases and the command
//!   catalogue published to the home command palette.
//! * [`infrastructure`] — PTY/session adapters.
//! * [`presenter`] — Iced-side rendering for the
//!   terminal workspace surface.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;

pub use application::TerminalCommands;
