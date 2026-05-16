use openspace_core::app_command::FeatureCommand;
use openspace_core::core_errors::CoreError;
use openspace_core::session::Session;

pub trait FeatureRuntime: Send {
    fn feature_id(&self) -> &str;
    fn on_session_activate(&mut self, session: &Session);
    fn on_session_deactivate(&mut self, session: &Session);
    fn handle_command(&mut self, cmd: &FeatureCommand) -> Result<(), CoreError>;
}

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
