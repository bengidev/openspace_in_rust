use crate::permission::PermissionProfile;
use crate::session::{SessionDescriptor, SessionMode};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageCommand {
    SaveSession { session_id: Uuid },
    LoadSession { session_id: Uuid },
    DeleteSession { session_id: Uuid },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FeatureCommand {
    Activate,
    Deactivate,
    Custom { payload: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppCommand {
    CreateSession {
        project_folder: PathBuf,
        descriptor: SessionDescriptor,
    },
    CloseSession {
        session_id: Uuid,
    },
    SwitchMode {
        session_id: Uuid,
        mode: SessionMode,
    },
    UpdatePermission {
        session_id: Uuid,
        profile: PermissionProfile,
    },
    OpenCommandPalette,
    DispatchToFeature {
        feature_id: String,
        command: FeatureCommand,
    },
    Storage(StorageCommand),
}
