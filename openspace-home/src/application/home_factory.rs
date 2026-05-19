//! Home stage factory.
//!
//! The home stage is the existing app shell presented as a stable
//! surface for the onboarding router. We do not subclass or wrap
//! the shell here; this module exists so the routing layer reads
//! consistently:
//!
//! ```text
//! onboarding-app -> openspace-welcome -> openspace-home   // first run
//! onboarding-app -> openspace-home                        // subsequent runs
//! ```
//!
//! Adding new behaviour at the home stage (for example, a "welcome
//! back" toast on the first session after onboarding) belongs here
//! rather than directly in the presenter.

use openspace_theme::tokens::ThemeMode;

use crate::presenter::AppShell;

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
