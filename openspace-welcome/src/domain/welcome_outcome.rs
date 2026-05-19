//! What the parent router needs to do after dispatching a welcome
//! message.
//!
//! Keeping the routing decision out of the welcome state lets the
//! reducer stay pure: tests can drive the welcome flow without ever
//! touching the filesystem or the host application.

use openspace_theme::tokens::ThemeMode;

/// Outcome the parent router needs after dispatching a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WelcomeOutcome {
    /// State updated locally; no transition.
    None,
    /// User toggled the theme; the parent should mirror that change
    /// into any shared theme state it owns.
    ThemeToggled(ThemeMode),
    /// User accepted the welcome window; the router should mark the
    /// persistence flag and transition to the home shell.
    Completed,
    /// User skipped. Treated identically to `Completed` for routing,
    /// but a separate variant lets the audit/event sink distinguish
    /// the two behaviours later if we want to.
    Skipped,
}
