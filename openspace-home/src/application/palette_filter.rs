//! Pure filter used by the command palette overlay.
//!
//! Substring matching against the command title and id, gated by
//! the active session's mode + permission profile. The filter is
//! pure so it can be exercised end-to-end from unit tests without
//! spinning up the Iced runtime.

use openspace_core::command_palette::CommandMetadata;
use openspace_core::session::Session;

/// Filter commands by active session context.
/// V1: substring matching on title.
pub fn filter_by_context_and_query<'a>(
    commands: &'a [CommandMetadata],
    session: Option<&Session>,
    query: &str,
) -> Vec<&'a CommandMetadata> {
    let lower_query = query.to_lowercase();
    commands
        .iter()
        .filter(|meta| matches_session(meta, session))
        .filter(|meta| {
            if lower_query.is_empty() {
                return true;
            }
            meta.title.to_lowercase().contains(&lower_query)
                || meta.id.0.to_lowercase().contains(&lower_query)
        })
        .collect()
}

fn matches_session(meta: &CommandMetadata, session: Option<&Session>) -> bool {
    let Some(session) = session else {
        // No session active: only commands with no mode requirement
        // are available.
        return meta.context.mode.is_none();
    };

    if let Some(required_mode) = &meta.context.mode
        && *required_mode != session.mode {
            return false;
        }

    if let Some(required_profile) = &meta.context.permission_profile
        && *required_profile != session.permission {
            return false;
        }

    true
}
