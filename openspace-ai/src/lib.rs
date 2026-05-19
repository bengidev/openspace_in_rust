//! `openspace-ai` — AI runtime feature crate.
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — prompt / response / ContextPack value types.
//! * [`application`] — streaming lifecycle and proposal
//!   normalisation.
//! * [`infrastructure`] — provider adapters.
//! * [`presenter`] — Iced-side AI affordances.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;
