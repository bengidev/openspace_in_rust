//! Messages produced by the welcome view.
//!
//! `EnterPressed` and `Skipped` are both terminal — they tell the
//! parent router to mark the welcome flag and transition to the
//! home shell. The router is the single place that owns transition
//! logic; the welcome view does not own it.

use std::time::Instant;

#[derive(Debug, Clone)]
pub enum WelcomeMessage {
    Tick(Instant),
    ToggleTheme,
    /// User started holding the mouse button down on the orb.
    /// Begins the zoom-in / speed-up ramp.
    OrbPressed,
    /// User released the mouse button. Begins the decay back to the
    /// rest state.
    OrbReleased,
    EnterPressed,
    Skipped,
}
