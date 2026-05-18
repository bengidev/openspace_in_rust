//! First-run welcome window.
//!
//! See `persistence.rs` for the sentinel-file flag that ensures the
//! window is shown only once, and `ascii_orb.rs` for the pixel
//! particle canvas program rendered in the welcome hero. The
//! welcome state + view module is added in a follow-up commit.

pub mod ascii_orb;
pub mod persistence;

pub use persistence::{
    FileWelcomePersistence, InMemoryWelcomePersistence, WelcomePersistence,
    WelcomePersistenceError,
};
