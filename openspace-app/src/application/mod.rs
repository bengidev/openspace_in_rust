//! Composition-root application layer.
//!
//! Owns the [`OnboardingApp`] router that drives the welcome →
//! home transition. The reducer is split out so the bootstrap
//! (`run` / `run_with`) and the presenter can both depend on it
//! without forming a cycle.

pub mod app_router;
pub mod onboarding_app;

pub use onboarding_app::{init, update, OnboardingApp};
