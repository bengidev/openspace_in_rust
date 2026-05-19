//! Terminal command catalogue published to the home command
//! palette.
//!
//! Built as a [`CommandDescriptorProvider`] so the home
//! [`CommandRegistry`] can merge in our shortcuts during shell
//! construction. Real PTY-driving commands plug into the runtime
//! manager via the infrastructure layer once they exist.

use openspace_core::command_palette::{
    CommandCategory, CommandContextRequirements, CommandDescriptorProvider, CommandMetadata,
    KeyboardShortcut, ShortcutModifiers,
};

pub struct TerminalCommands;

impl CommandDescriptorProvider for TerminalCommands {
    fn command_descriptors() -> Vec<CommandMetadata> {
        vec![
            CommandMetadata::new(
                "terminal.new_tab",
                "New Terminal Tab",
                CommandCategory::Terminal,
            )
            .with_context(CommandContextRequirements {
                mode: Some(openspace_core::session::SessionMode::Terminal),
                ..Default::default()
            })
            .with_shortcut(KeyboardShortcut::new(
                "t",
                ShortcutModifiers {
                    ctrl: true,
                    shift: false,
                    alt: false,
                    meta: false,
                },
            )),
            CommandMetadata::new(
                "terminal.clear",
                "Clear Terminal",
                CommandCategory::Terminal,
            )
            .with_context(CommandContextRequirements {
                mode: Some(openspace_core::session::SessionMode::Terminal),
                ..Default::default()
            }),
            CommandMetadata::new(
                "terminal.kill_process",
                "Kill Active Process",
                CommandCategory::Terminal,
            )
            .with_context(CommandContextRequirements {
                mode: Some(openspace_core::session::SessionMode::Terminal),
                permission_profile: Some(openspace_core::permission::PermissionProfile::FullAccess),
                ..Default::default()
            }),
        ]
    }
}
