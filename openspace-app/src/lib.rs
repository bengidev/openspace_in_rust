//! `openspace-app` — the desktop application's composition root.
//!
//! Mounts the welcome → home routing pipeline. The actual feature
//! crates own their own state, view, and persistence; this crate
//! only wires them together and runs the Iced event loop.
//!
//! ```text
//! openspace-app
//!  ├── openspace-welcome   (first-run welcome window)
//!  └── openspace-home      (post-welcome workspace shell)
//! ```
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — pure types: window sizing constants, the
//!   stage enum, the top-level message envelope.
//! * [`application`] — the `OnboardingApp` router state and
//!   reducer.
//! * [`presenter`] — Iced view + subscription dispatcher,
//!   plus the debug-only floating overlay.
//! * [`infrastructure`] — Iced bootstrap that loads the
//!   filesystem-backed welcome persistence and runs the event
//!   loop.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;
