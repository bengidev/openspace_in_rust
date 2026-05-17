use openspace_core::command_palette::{CommandId, CommandMetadata, KeyboardShortcut};
use std::collections::HashMap;

/// Merged registry of commands from all feature crates.
#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    commands: HashMap<CommandId, CommandMetadata>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge feature descriptors into the registry.
    /// Logs warnings for duplicate command IDs (overwrites with latest).
    pub fn merge(&mut self, descriptors: Vec<CommandMetadata>) {
        for meta in descriptors {
            if self.commands.contains_key(&meta.id) {
                tracing::warn!("duplicate command id overwritten: {:?}", meta.id);
            }
            self.commands.insert(meta.id.clone(), meta);
        }
    }

    /// Detect shortcut conflicts: same shortcut in same context.
    /// Returns conflicting pairs.
    pub fn detect_conflicts(&self) -> Vec<(CommandId, CommandId)> {
        let mut by_shortcut: HashMap<Option<&KeyboardShortcut>, Vec<&CommandMetadata>> =
            HashMap::new();
        for meta in self.commands.values() {
            by_shortcut
                .entry(meta.shortcut.as_ref())
                .or_default()
                .push(meta);
        }

        let mut conflicts = Vec::new();
        for group in by_shortcut.values() {
            for i in 0..group.len() {
                for j in (i + 1)..group.len() {
                    let a = group[i];
                    let b = group[j];
                    if a.shortcut.is_some() && Self::same_context(&a.context, &b.context) {
                        conflicts.push((a.id.clone(), b.id.clone()));
                    }
                }
            }
        }
        conflicts
    }

    fn same_context(
        a: &openspace_core::command_palette::CommandContextRequirements,
        b: &openspace_core::command_palette::CommandContextRequirements,
    ) -> bool {
        a.mode == b.mode
            && a.permission_profile == b.permission_profile
            && a.focus == b.focus
            && a.selection == b.selection
            && a.project_state == b.project_state
    }

    pub fn all(&self) -> Vec<&CommandMetadata> {
        self.commands.values().collect()
    }

    pub fn get(&self, id: &CommandId) -> Option<&CommandMetadata> {
        self.commands.get(id)
    }
}
