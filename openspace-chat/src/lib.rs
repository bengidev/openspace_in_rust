//! `openspace-chat` — chat workflow feature crate.
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — pure chat value types (messages, threads).
//! * [`application`] — chat lifecycle + command catalogue.
//! * [`infrastructure`] — provider streams + persistence.
//! * [`presenter`] — Iced-side rendering for the chat
//!   workflow surface.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;

pub use application::ChatCommands;
