//! Welcome feature application layer.
//!
//! Owns the [`WelcomeState`] reducer and the pure animation curves
//! that derive UI inputs from user intent. The reducer is the
//! single place that mutates state in response to messages;
//! persistence side-effects are surfaced as [`WelcomeOutcome`]s and
//! are executed by the host router.

pub mod welcome_dynamics;
pub mod welcome_messages;
pub mod welcome_state;

pub use welcome_dynamics::dynamics_for_progress;
pub use welcome_messages::WelcomeMessage;
pub use welcome_state::{WelcomeState, mark_completed};
