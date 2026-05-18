//! Home shell — the post-welcome workspace.
//!
//! The home stage is the existing [`AppShell`] presented as a stable
//! surface for the onboarding router. We do not subclass or wrap the
//! shell here; this module exists so the routing layer reads
//! consistently:
//!
//! ```text
//! onboarding-app -> onboarding-welcome -> onboarding-home   // first run
//! onboarding-app -> onboarding-home                         // subsequent runs
//! ```
//!
//! Adding new behaviour at the home stage (for example, a "welcome
//! back" toast on the first session after onboarding) belongs here
//! rather than in `app_shell` directly.

pub use crate::app_shell::{
    AppShell, Message as HomeMessage, shell_subscription as subscription,
    shell_update as update, shell_view as view,
};

use openspace_theme::tokens::ThemeMode;

/// Constructs the home stage with the given theme mode.
///
/// Surfaces a small wrapper over [`AppShell::with_theme_mode`] so
/// callers do not have to know the underlying type to instantiate
/// the home stage.
pub fn new(theme_mode: ThemeMode) -> AppShell {
    AppShell::with_theme_mode(theme_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_requested_theme_mode() {
        let shell = new(ThemeMode::Light);
        assert_eq!(shell.theme_mode(), ThemeMode::Light);

        let shell = new(ThemeMode::Dark);
        assert_eq!(shell.theme_mode(), ThemeMode::Dark);
    }
}
