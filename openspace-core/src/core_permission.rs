use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionProfile {
    Default,
    AutoReview,
    FullAccess,
    Custom { rules: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionDecision {
    Allow,
    Deny,
    Review,
}

impl PermissionProfile {
    pub fn can_execute(&self, action: &str) -> PermissionDecision {
        match self {
            PermissionProfile::FullAccess => PermissionDecision::Allow,
            PermissionProfile::Default => {
                if action.starts_with("read") || action.starts_with("view") {
                    PermissionDecision::Allow
                } else {
                    PermissionDecision::Deny
                }
            }
            PermissionProfile::AutoReview => PermissionDecision::Review,
            PermissionProfile::Custom { rules } => {
                if rules.iter().any(|r| action.contains(r)) {
                    PermissionDecision::Allow
                } else {
                    PermissionDecision::Deny
                }
            }
        }
    }
}
