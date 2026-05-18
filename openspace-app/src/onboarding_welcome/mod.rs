//! First-run welcome window.
//!
//! See `persistence.rs` for the sentinel-file flag that ensures the
//! window is shown only once. Additional submodules (the welcome
//! state + view, and the ASCII particle orb canvas program) are
//! introduced in follow-up commits.

pub mod persistence;

pub use persistence::{
    FileWelcomePersistence, InMemoryWelcomePersistence, WelcomePersistence,
    WelcomePersistenceError,
};
