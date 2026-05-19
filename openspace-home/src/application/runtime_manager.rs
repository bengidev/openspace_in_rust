//! Runtime manager — owns the registered feature runtimes and
//! routes activation/deactivation/dispatch through them.
//!
//! Sessions trigger lifecycle hooks (`on_session_activate` /
//! `on_session_deactivate`) and feature commands flow through
//! [`RuntimeManager::dispatch`]. The router is the single caller
//! of these entry points, so feature lifecycle stays observable
//! from one place.

use openspace_core::app_command::FeatureCommand;
use openspace_core::core_errors::CoreError;
use openspace_core::session::Session;

use crate::domain::FeatureRuntime;

#[derive(Default)]
pub struct RuntimeManager {
    runtimes: Vec<Box<dyn FeatureRuntime>>,
}

impl std::fmt::Debug for RuntimeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeManager")
            .field("runtimes", &self.runtimes.len())
            .finish()
    }
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {
            runtimes: Vec::new(),
        }
    }

    pub fn register(&mut self, runtime: Box<dyn FeatureRuntime>) {
        self.runtimes.push(runtime);
    }

    pub fn activate_for_session(&mut self, session: &Session) {
        for runtime in &mut self.runtimes {
            runtime.on_session_activate(session);
        }
    }

    pub fn deactivate_for_session(&mut self, session: &Session) {
        for runtime in &mut self.runtimes {
            runtime.on_session_deactivate(session);
        }
    }

    pub fn dispatch(&mut self, feature_id: &str, cmd: &FeatureCommand) -> Result<(), CoreError> {
        self.runtimes
            .iter_mut()
            .find(|r| r.feature_id() == feature_id)
            .ok_or_else(|| CoreError::FeatureNotFound(feature_id.to_string()))?
            .handle_command(cmd)
    }
}
