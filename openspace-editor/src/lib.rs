pub mod editor_buffers;
pub mod editor_documents;

use openspace_core::command_palette::{
    CommandCategory, CommandContextRequirements, CommandDescriptorProvider, CommandMetadata,
    KeyboardShortcut, ShortcutModifiers,
};

pub struct EditorCommands;

impl CommandDescriptorProvider for EditorCommands {
    fn command_descriptors() -> Vec<CommandMetadata> {
        vec![
            CommandMetadata::new("editor.new_file", "New File", CommandCategory::Editor)
                .with_context(CommandContextRequirements {
                    mode: Some(openspace_core::session::SessionMode::Editor),
                    ..Default::default()
                })
                .with_shortcut(KeyboardShortcut::new(
                    "n",
                    ShortcutModifiers {
                        ctrl: true,
                        shift: true,
                        alt: false,
                        meta: false,
                    },
                )),
            CommandMetadata::new("editor.save", "Save File", CommandCategory::Editor)
                .with_context(CommandContextRequirements {
                    mode: Some(openspace_core::session::SessionMode::Editor),
                    focus: Some("editor".to_string()),
                    ..Default::default()
                })
                .with_shortcut(KeyboardShortcut::new(
                    "s",
                    ShortcutModifiers {
                        ctrl: true,
                        shift: false,
                        alt: false,
                        meta: false,
                    },
                )),
            CommandMetadata::new("editor.find", "Find in File", CommandCategory::Editor)
                .with_context(CommandContextRequirements {
                    mode: Some(openspace_core::session::SessionMode::Editor),
                    focus: Some("editor".to_string()),
                    ..Default::default()
                })
                .with_shortcut(KeyboardShortcut::new(
                    "f",
                    ShortcutModifiers {
                        ctrl: true,
                        shift: false,
                        alt: false,
                        meta: false,
                    },
                )),
        ]
    }
}
