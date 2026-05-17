use crate::permission::PermissionProfile;
use crate::session::SessionMode;

/// Unique identifier for a command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(pub String);

impl From<String> for CommandId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for CommandId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Category used to group commands in the palette UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Navigation,
    Session,
    Terminal,
    Chat,
    Editor,
    File,
    Git,
    Settings,
    Help,
}

/// Modifier keys for a keyboard shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortcutModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl ShortcutModifiers {
    pub const fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }
}

/// Keyboard shortcut descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardShortcut {
    pub key: String,
    pub modifiers: ShortcutModifiers,
}

impl KeyboardShortcut {
    pub fn new(key: impl Into<String>, modifiers: ShortcutModifiers) -> Self {
        Self {
            key: key.into(),
            modifiers,
        }
    }
}

/// Context requirements that must be satisfied for a command to be available.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct CommandContextRequirements {
    /// Required session mode, if any.
    pub mode: Option<SessionMode>,
    /// Required permission profile, if any.
    pub permission_profile: Option<PermissionProfile>,
    /// Required focus area (e.g., "editor", "terminal"), if any.
    pub focus: Option<String>,
    /// Required selection state (e.g., "text_selected"), if any.
    pub selection: Option<String>,
    /// Required project state (e.g., "project_open"), if any.
    pub project_state: Option<String>,
}

/// Metadata describing a command that can appear in the palette.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandMetadata {
    pub id: CommandId,
    pub title: String,
    pub category: CommandCategory,
    pub context: CommandContextRequirements,
    pub shortcut: Option<KeyboardShortcut>,
}

impl CommandMetadata {
    pub fn new(
        id: impl Into<CommandId>,
        title: impl Into<String>,
        category: CommandCategory,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category,
            context: CommandContextRequirements::default(),
            shortcut: None,
        }
    }

    pub fn with_context(mut self, context: CommandContextRequirements) -> Self {
        self.context = context;
        self
    }

    pub fn with_shortcut(mut self, shortcut: KeyboardShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }
}

/// Trait for feature crates to expose their command descriptors.
pub trait CommandDescriptorProvider {
    fn command_descriptors() -> Vec<CommandMetadata>;
}
