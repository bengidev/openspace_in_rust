//! Welcome feature domain layer.
//!
//! The domain layer holds pure contracts only. No Iced, no
//! filesystem, no allocation strategy beyond what the trait shape
//! demands. Application and infrastructure layers depend inward on
//! these types.

pub mod welcome_outcome;
pub mod welcome_persistence;

pub use welcome_outcome::WelcomeOutcome;
pub use welcome_persistence::{WelcomePersistence, WelcomePersistenceError};
