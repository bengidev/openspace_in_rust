use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::permission::{PermissionDecision, PermissionProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionMode {
    Terminal,
    Chat,
    Editor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionDescriptor {
    pub name: String,
    pub created_at: SystemTime,
}

impl SessionDescriptor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            created_at: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub mode: SessionMode,
    pub permission: PermissionProfile,
    pub project_folder: PathBuf,
    pub descriptor: SessionDescriptor,
}

impl Session {
    pub fn new(project_folder: PathBuf, descriptor: SessionDescriptor) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode: SessionMode::Terminal,
            permission: PermissionProfile::Default,
            project_folder,
            descriptor,
        }
    }

    pub fn can_execute(&self, action: &str) -> PermissionDecision {
        self.permission.can_execute(action)
    }
}
