//! Feature runtime contract.
//!
//! The [`FeatureRuntime`] trait describes the surface a feature
//! must expose to plug into the home stage's runtime manager.
//! Sessions activate / deactivate features as the user switches
//! modes, and feature-specific commands flow through `dispatch`.
//!
//! Kept in the domain layer because both the application layer
//! (the runtime manager) and the infrastructure layer (mock
//! runtime, real feature runtimes) depend inward on this trait.

use openspace_core::app_command::FeatureCommand;
use openspace_core::core_errors::CoreError;
use openspace_core::session::Session;

pub trait FeatureRuntime: Send {
    fn feature_id(&self) -> &str;
    fn on_session_activate(&mut self, session: &Session);
    fn on_session_deactivate(&mut self, session: &Session);
    fn handle_command(&mut self, cmd: &FeatureCommand) -> Result<(), CoreError>;
}
