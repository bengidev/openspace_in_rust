use std::sync::{Arc, Mutex};

use openspace_core::app_command::FeatureCommand;
use openspace_core::core_errors::CoreError;
use openspace_core::session::Session;
use uuid::Uuid;

use crate::feature_runtime::FeatureRuntime;

#[derive(Debug, Clone, PartialEq)]
pub struct MockFeatureState {
    pub activate_count: usize,
    pub deactivate_count: usize,
    pub command_count: usize,
    pub last_session_id: Option<Uuid>,
    pub last_command: Option<FeatureCommand>,
}

#[derive(Clone)]
pub struct MockFeatureRuntime {
    feature_id: String,
    state: Arc<Mutex<MockFeatureState>>,
}

impl MockFeatureRuntime {
    pub fn new(feature_id: impl Into<String>) -> Self {
        Self {
            feature_id: feature_id.into(),
            state: Arc::new(Mutex::new(MockFeatureState {
                activate_count: 0,
                deactivate_count: 0,
                command_count: 0,
                last_session_id: None,
                last_command: None,
            })),
        }
    }

    pub fn state(&self) -> MockFeatureState {
        self.state.lock().unwrap().clone()
    }
}

impl FeatureRuntime for MockFeatureRuntime {
    fn feature_id(&self) -> &str {
        &self.feature_id
    }

    fn on_session_activate(&mut self, session: &Session) {
        let mut s = self.state.lock().unwrap();
        s.activate_count += 1;
        s.last_session_id = Some(session.id);
    }

    fn on_session_deactivate(&mut self, session: &Session) {
        let mut s = self.state.lock().unwrap();
        s.deactivate_count += 1;
        s.last_session_id = Some(session.id);
    }

    fn handle_command(&mut self, cmd: &FeatureCommand) -> Result<(), CoreError> {
        let mut s = self.state.lock().unwrap();
        s.command_count += 1;
        s.last_command = Some(cmd.clone());
        Ok(())
    }
}
