use crate::core_errors::CoreError;
use crate::permission::PermissionProfile;
use crate::session::SessionMode;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureLifecycleState {
    Starting,
    Ready,
    Paused,
    Stopping,
    Stopped,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    SessionCreated { session_id: Uuid },
    SessionClosed { session_id: Uuid },
    ModeChanged { session_id: Uuid, new_mode: SessionMode },
    PermissionChanged { session_id: Uuid, new_profile: PermissionProfile },
    FeatureLifecycle { feature_id: String, state: FeatureLifecycleState },
    Audit { action: String, session_id: Uuid },
    Error { session_id: Option<Uuid>, error: CoreError },
}
