//! Welcome feature infrastructure layer.
//!
//! Concrete persistence backends. The filesystem-backed store is
//! the production default; the in-memory variant is used by tests
//! and previews.

pub mod welcome_file_persistence;
pub mod welcome_memory_persistence;

pub use welcome_file_persistence::FileWelcomePersistence;
pub use welcome_memory_persistence::InMemoryWelcomePersistence;
