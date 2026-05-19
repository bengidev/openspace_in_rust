//! Home feature domain layer.
//!
//! Pure contracts shared across the home stage: layout constants
//! used by the shell view and the routing rules to keep the panels
//! within sane bounds, and the [`FeatureRuntime`] trait — the
//! contract every feature implements when it plugs into the
//! runtime manager.

pub mod feature_runtime;
pub mod layout;

pub use feature_runtime::FeatureRuntime;
