//! First-run welcome window.
//!
//! See `welcome.rs` for the state + view, `ascii_orb.rs` for the
//! pixel-particle canvas program, and `persistence.rs` for the
//! sentinel-file flag that ensures the window is shown only once.

pub mod ascii_orb;
pub mod persistence;
pub mod welcome;

pub use persistence::{
    FileWelcomePersistence, InMemoryWelcomePersistence, WelcomePersistence,
    WelcomePersistenceError,
};
pub use welcome::{WelcomeMessage, WelcomeOutcome, WelcomeState, view};
