//! `openspace-welcome` — first-run welcome window feature crate.
//!
//! Layered along Clean Architecture lines so the welcome page can be
//! reasoned about and tested in isolation:
//!
//! * [`domain`] — pure contracts: outcomes, persistence trait,
//!   persistence error type. No I/O, no Iced, no allocation policy.
//! * [`application`] — the [`WelcomeState`] reducer and the
//!   pure dynamics curve driving the orb animation.
//! * [`infrastructure`] — concrete persistence backends
//!   (filesystem-backed for production, in-memory for tests).
//! * [`presenter`] — Iced view + canvas program. The host app
//!   maps these messages into its own router.
//!
//! Re-exports below provide a flat public surface for the
//! composition root, while the inner layers stay accessible for
//! tests and adapters.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;

pub use application::{
    WelcomeMessage, WelcomeState, dynamics_for_progress, mark_completed,
};
pub use domain::{WelcomeOutcome, WelcomePersistence, WelcomePersistenceError};
pub use infrastructure::{FileWelcomePersistence, InMemoryWelcomePersistence};
pub use presenter::view;
