//! Chat command catalogue published to the home command palette.

use openspace_core::command_palette::{
    CommandCategory, CommandContextRequirements, CommandDescriptorProvider, CommandMetadata,
    KeyboardShortcut, ShortcutModifiers,
};

pub struct ChatCommands;

impl CommandDescriptorProvider for ChatCommands {
    fn command_descriptors() -> Vec<CommandMetadata> {
        vec![
            CommandMetadata::new("chat.new_thread", "New Chat Thread", CommandCategory::Chat)
                .with_context(CommandContextRequirements {
                    mode: Some(openspace_core::session::SessionMode::Chat),
                    ..Default::default()
                })
                .with_shortcut(KeyboardShortcut::new(
                    "n",
                    ShortcutModifiers {
                        ctrl: true,
                        shift: false,
                        alt: false,
                        meta: false,
                    },
                )),
            CommandMetadata::new(
                "chat.clear_history",
                "Clear Chat History",
                CommandCategory::Chat,
            )
            .with_context(CommandContextRequirements {
                mode: Some(openspace_core::session::SessionMode::Chat),
                ..Default::default()
            }),
            CommandMetadata::new("chat.export", "Export Chat", CommandCategory::Chat).with_context(
                CommandContextRequirements {
                    mode: Some(openspace_core::session::SessionMode::Chat),
                    permission_profile: Some(
                        openspace_core::permission::PermissionProfile::Default,
                    ),
                    ..Default::default()
                },
            ),
        ]
    }
}
