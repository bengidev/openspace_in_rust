//! Composition-root domain layer.
//!
//! Pure types for the onboarding router: window sizing rules, the
//! [`Stage`] enum tagging which sub-feature is active, and the
//! top-level [`Message`] envelope routed into Iced.

pub mod app_messages;
pub mod app_stage;
pub mod app_window_sizes;

pub use app_messages::Message;
pub use app_stage::Stage;
pub use app_window_sizes::{
    HOME_DEFAULT_SIZE, HOME_MIN_SIZE, WELCOME_DEFAULT_SIZE, WELCOME_MIN_SIZE,
};
