use std::collections::HashMap;

use openspace_core::app_command::{AppCommand, FeatureCommand, StorageCommand};
use openspace_core::app_event::{AppEvent, FeatureLifecycleState};
use openspace_core::core_errors::CoreError;
use openspace_core::session::Session;
use uuid::Uuid;

use crate::feature_runtime::RuntimeManager;

#[derive(Debug, Default)]
pub struct AppRouter {
    sessions: HashMap<Uuid, Session>,
    active_session_id: Option<Uuid>,
    pub runtime_manager: RuntimeManager,
}

impl AppRouter {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
            runtime_manager: RuntimeManager::new(),
        }
    }

    pub fn active_session(&self) -> Option<&Session> {
        self.active_session_id.and_then(|id| self.sessions.get(&id))
    }

    pub fn active_session_mut(&mut self) -> Option<&mut Session> {
        self.active_session_id.and_then(|id| self.sessions.get_mut(&id))
    }

    pub fn sessions(&self) -> &HashMap<Uuid, Session> {
        &self.sessions
    }

    pub fn apply(&mut self, command: AppCommand) -> Vec<AppEvent> {
        let mut events = Vec::new();

        match command {
            AppCommand::CreateSession {
                project_folder,
                descriptor,
            } => {
                let session = Session::new(project_folder, descriptor);
                let id = session.id;
                self.sessions.insert(id, session);
                self.active_session_id = Some(id);
                events.push(AppEvent::SessionCreated { session_id: id });
                events.push(AppEvent::Audit {
                    action: "session_created".to_string(),
                    session_id: id,
                });
            }
            AppCommand::CloseSession { session_id } => {
                if let Some(session) = self.sessions.remove(&session_id) {
                    self.runtime_manager.deactivate_for_session(&session);
                    if self.active_session_id == Some(session_id) {
                        self.active_session_id = self.sessions.keys().next().copied();
                    }
                    events.push(AppEvent::SessionClosed { session_id });
                    events.push(AppEvent::Audit {
                        action: "session_closed".to_string(),
                        session_id,
                    });
                } else {
                    events.push(AppEvent::Error {
                        session_id: Some(session_id),
                        error: CoreError::SessionNotFound(session_id.to_string()),
                    });
                }
            }
            AppCommand::SwitchMode { session_id, mode } => {
                if let Some(existing) = self.sessions.get(&session_id) {
                    if existing.mode != mode {
                        if let Some(active_id) = self.active_session_id {
                            if active_id == session_id {
                                if let Some(s) = self.sessions.get(&active_id) {
                                    self.runtime_manager.deactivate_for_session(s);
                                }
                            }
                        }
                        if let Some(session) = self.sessions.get_mut(&session_id) {
                            session.mode = mode.clone();
                            events.push(AppEvent::ModeChanged {
                                session_id,
                                new_mode: mode.clone(),
                            });
                        }
                        if let Some(s) = self.sessions.get(&session_id) {
                            self.runtime_manager.activate_for_session(s);
                        }
                    }
                } else {
                    events.push(AppEvent::Error {
                        session_id: Some(session_id),
                        error: CoreError::SessionNotFound(session_id.to_string()),
                    });
                }
            }
            AppCommand::UpdatePermission { session_id, profile } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.permission = profile.clone();
                    events.push(AppEvent::PermissionChanged {
                        session_id,
                        new_profile: profile,
                    });
                } else {
                    events.push(AppEvent::Error {
                        session_id: Some(session_id),
                        error: CoreError::SessionNotFound(session_id.to_string()),
                    });
                }
            }
            AppCommand::OpenCommandPalette => {
                events.push(AppEvent::Audit {
                    action: "command_palette_opened".to_string(),
                    session_id: self.active_session_id.unwrap_or_else(|| Uuid::from_u128(0)),
                });
            }
            AppCommand::DispatchToFeature {
                feature_id,
                command,
            } => {
                match self.runtime_manager.dispatch(&feature_id, &command) {
                    Ok(()) => {
                        events.push(AppEvent::FeatureLifecycle {
                            feature_id,
                            state: match command {
                                FeatureCommand::Activate => FeatureLifecycleState::Ready,
                                FeatureCommand::Deactivate => FeatureLifecycleState::Stopped,
                                _ => FeatureLifecycleState::Ready,
                            },
                        });
                    }
                    Err(e) => {
                        events.push(AppEvent::Error {
                            session_id: self.active_session_id,
                            error: e,
                        });
                    }
                }
            }
            AppCommand::Storage(cmd) => match cmd {
                StorageCommand::SaveSession { session_id } => {
                    events.push(AppEvent::Audit {
                        action: "session_saved".to_string(),
                        session_id,
                    });
                }
                StorageCommand::LoadSession { session_id } => {
                    events.push(AppEvent::Audit {
                        action: "session_loaded".to_string(),
                        session_id,
                    });
                }
                StorageCommand::DeleteSession { session_id } => {
                    events.push(AppEvent::Audit {
                        action: "session_deleted".to_string(),
                        session_id,
                    });
                }
            },
        }

        events
    }
}
