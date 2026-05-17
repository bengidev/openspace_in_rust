use openspace_app::command_palette::filter::filter_by_context_and_query;
use openspace_app::command_palette::{CommandPaletteOverlay, CommandRegistry, PaletteMessage};
use openspace_core::command_palette::{
    CommandCategory, CommandContextRequirements, CommandId, CommandMetadata, KeyboardShortcut,
    ShortcutModifiers,
};
use openspace_core::permission::PermissionProfile;
use openspace_core::session::{Session, SessionDescriptor, SessionMode};
use std::path::PathBuf;

fn make_meta(id: &str, title: &str, category: CommandCategory) -> CommandMetadata {
    CommandMetadata::new(id, title, category)
}

fn make_meta_with_context(
    id: &str,
    title: &str,
    category: CommandCategory,
    context: CommandContextRequirements,
) -> CommandMetadata {
    CommandMetadata::new(id, title, category).with_context(context)
}

fn make_meta_with_shortcut(
    id: &str,
    title: &str,
    category: CommandCategory,
    shortcut: KeyboardShortcut,
) -> CommandMetadata {
    CommandMetadata::new(id, title, category).with_shortcut(shortcut)
}

#[test]
fn registry_merge_adds_commands() {
    let mut reg = CommandRegistry::new();
    let descs = vec![
        make_meta("a", "Alpha", CommandCategory::Navigation),
        make_meta("b", "Beta", CommandCategory::Session),
    ];
    reg.merge(descs);
    let all = reg.all();
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|m| m.id == CommandId("a".to_string())));
    assert!(all.iter().any(|m| m.id == CommandId("b".to_string())));
}

#[test]
fn registry_merge_overwrites_duplicates() {
    let mut reg = CommandRegistry::new();
    reg.merge(vec![make_meta("a", "Alpha", CommandCategory::Navigation)]);
    reg.merge(vec![make_meta("a", "Alpha Updated", CommandCategory::Session)]);
    let all = reg.all();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "Alpha Updated");
    assert_eq!(all[0].category, CommandCategory::Session);
}

#[test]
fn registry_conflict_detection_finds_same_shortcut_same_context() {
    let mut reg = CommandRegistry::new();
    let shortcut = KeyboardShortcut::new("k", ShortcutModifiers::none());
    let descs = vec![
        make_meta_with_shortcut("a", "Alpha", CommandCategory::Navigation, shortcut.clone()),
        make_meta_with_shortcut("b", "Beta", CommandCategory::Session, shortcut.clone()),
    ];
    reg.merge(descs);
    let conflicts = reg.detect_conflicts();
    assert_eq!(conflicts.len(), 1);
    let (ref a, ref b) = conflicts[0];
    assert!((a.0 == "a" && b.0 == "b") || (a.0 == "b" && b.0 == "a"));
}

#[test]
fn registry_conflict_detection_ignores_different_shortcuts() {
    let mut reg = CommandRegistry::new();
    let descs = vec![
        make_meta_with_shortcut(
            "a",
            "Alpha",
            CommandCategory::Navigation,
            KeyboardShortcut::new("k", ShortcutModifiers::none()),
        ),
        make_meta_with_shortcut(
            "b",
            "Beta",
            CommandCategory::Session,
            KeyboardShortcut::new("j", ShortcutModifiers::none()),
        ),
    ];
    reg.merge(descs);
    let conflicts = reg.detect_conflicts();
    assert!(conflicts.is_empty());
}

#[test]
fn registry_conflict_detection_ignores_same_shortcut_different_context() {
    let mut reg = CommandRegistry::new();
    let shortcut = KeyboardShortcut::new("k", ShortcutModifiers::none());
    let descs = vec![
        make_meta_with_shortcut("a", "Alpha", CommandCategory::Navigation, shortcut.clone())
            .with_context(CommandContextRequirements {
                mode: Some(SessionMode::Terminal),
                ..Default::default()
            }),
        make_meta_with_shortcut("b", "Beta", CommandCategory::Session, shortcut.clone())
            .with_context(CommandContextRequirements {
                mode: Some(SessionMode::Chat),
                ..Default::default()
            }),
    ];
    reg.merge(descs);
    let conflicts = reg.detect_conflicts();
    assert!(conflicts.is_empty());
}

#[test]
fn filter_by_context_no_session_allows_modeless_only() {
    let commands = vec![
        make_meta_with_context(
            "a",
            "Alpha",
            CommandCategory::Navigation,
            CommandContextRequirements::default(),
        ),
        make_meta_with_context(
            "b",
            "Beta",
            CommandCategory::Terminal,
            CommandContextRequirements {
                mode: Some(SessionMode::Terminal),
                ..Default::default()
            },
        ),
    ];
    let filtered = filter_by_context_and_query(&commands, None, "");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, CommandId("a".to_string()));
}

#[test]
fn filter_by_context_with_session_matches_mode() {
    let commands = vec![
        make_meta_with_context(
            "a",
            "Alpha",
            CommandCategory::Terminal,
            CommandContextRequirements {
                mode: Some(SessionMode::Terminal),
                ..Default::default()
            },
        ),
        make_meta_with_context(
            "b",
            "Beta",
            CommandCategory::Chat,
            CommandContextRequirements {
                mode: Some(SessionMode::Chat),
                ..Default::default()
            },
        ),
    ];
    let session = Session::new(
        PathBuf::from("."),
        SessionDescriptor::new("test"),
    );
    // session defaults to Terminal mode
    let filtered = filter_by_context_and_query(&commands, Some(&session), "");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, CommandId("a".to_string()));
}

#[test]
fn filter_by_context_with_permission_profile() {
    let commands = vec![
        make_meta_with_context(
            "a",
            "Alpha",
            CommandCategory::Terminal,
            CommandContextRequirements {
                mode: Some(SessionMode::Terminal),
                permission_profile: Some(PermissionProfile::FullAccess),
                ..Default::default()
            },
        ),
        make_meta_with_context(
            "b",
            "Beta",
            CommandCategory::Terminal,
            CommandContextRequirements {
                mode: Some(SessionMode::Terminal),
                permission_profile: Some(PermissionProfile::Default),
                ..Default::default()
            },
        ),
    ];
    let mut session = Session::new(PathBuf::from("."), SessionDescriptor::new("test"));
    session.permission = PermissionProfile::FullAccess;
    let filtered = filter_by_context_and_query(&commands, Some(&session), "");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, CommandId("a".to_string()));
}

#[test]
fn filter_by_query_substring_match() {
    let commands = vec![
        make_meta("alpha", "Alpha Command", CommandCategory::Navigation),
        make_meta("beta", "Beta Command", CommandCategory::Session),
    ];
    let filtered = filter_by_context_and_query(&commands, None, "alp");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, CommandId("alpha".to_string()));
}

#[test]
fn filter_by_query_empty_returns_all() {
    let commands = vec![
        make_meta("a", "Alpha", CommandCategory::Navigation),
        make_meta("b", "Beta", CommandCategory::Session),
    ];
    let filtered = filter_by_context_and_query(&commands, None, "");
    assert_eq!(filtered.len(), 2);
}

#[test]
fn overlay_open_sets_visible_and_clears_query() {
    let mut palette = CommandPaletteOverlay::default();
    palette.query = "foo".to_string();
    palette.open();
    assert!(palette.visible);
    assert!(palette.query.is_empty());
    assert_eq!(palette.selected_index, 0);
}

#[test]
fn overlay_close_hides_and_clears() {
    let mut palette = CommandPaletteOverlay::default();
    palette.open();
    palette.query = "bar".to_string();
    palette.filtered = vec![make_meta("a", "Alpha", CommandCategory::Navigation)];
    palette.close();
    assert!(!palette.visible);
    assert!(palette.query.is_empty());
    assert!(palette.filtered.is_empty());
}

#[test]
fn overlay_select_next_moves_index() {
    let mut palette = CommandPaletteOverlay::default();
    palette.filtered = vec![
        make_meta("a", "Alpha", CommandCategory::Navigation),
        make_meta("b", "Beta", CommandCategory::Session),
        make_meta("c", "Gamma", CommandCategory::Terminal),
    ];
    palette.selected_index = 0;
    palette.update(PaletteMessage::SelectNext, &[]);
    assert_eq!(palette.selected_index, 1);
    palette.update(PaletteMessage::SelectNext, &[]);
    assert_eq!(palette.selected_index, 2);
    palette.update(PaletteMessage::SelectNext, &[]);
    // should stay at last index
    assert_eq!(palette.selected_index, 2);
}

#[test]
fn overlay_select_previous_moves_index() {
    let mut palette = CommandPaletteOverlay::default();
    palette.filtered = vec![
        make_meta("a", "Alpha", CommandCategory::Navigation),
        make_meta("b", "Beta", CommandCategory::Session),
    ];
    palette.selected_index = 1;
    palette.update(PaletteMessage::SelectPrevious, &[]);
    assert_eq!(palette.selected_index, 0);
    palette.update(PaletteMessage::SelectPrevious, &[]);
    // should stay at 0
    assert_eq!(palette.selected_index, 0);
}

#[test]
fn overlay_confirm_with_no_selection_does_nothing() {
    let mut palette = CommandPaletteOverlay::default();
    palette.open();
    palette.filtered = vec![];
    palette.update(PaletteMessage::Confirm, &[]);
    // palette should remain open because nothing to confirm
    assert!(palette.visible);
}

#[test]
fn overlay_confirm_closes_palette() {
    let mut palette = CommandPaletteOverlay::default();
    palette.open();
    palette.filtered = vec![make_meta("a", "Alpha", CommandCategory::Navigation)];
    palette.update(PaletteMessage::Confirm, &[]);
    assert!(!palette.visible);
}

#[test]
fn feature_crates_expose_descriptors() {
    use openspace_chat::ChatCommands;
    use openspace_core::command_palette::CommandDescriptorProvider;
    use openspace_editor::EditorCommands;
    use openspace_terminal::TerminalCommands;

    let chat = ChatCommands::command_descriptors();
    let editor = EditorCommands::command_descriptors();
    let terminal = TerminalCommands::command_descriptors();

    assert!(!chat.is_empty(), "chat descriptors should not be empty");
    assert!(!editor.is_empty(), "editor descriptors should not be empty");
    assert!(!terminal.is_empty(), "terminal descriptors should not be empty");
}
