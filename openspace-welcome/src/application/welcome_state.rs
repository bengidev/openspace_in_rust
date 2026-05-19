//! Welcome page reducer.
//!
//! State of the welcome page.
//!
//! The animation timeline is captured by `(started_at, now)`. Both
//! are monotonic `Instant`s so the orb canvas can run a pure draw
//! based on elapsed seconds without ever calling `Instant::now()` in
//! the render path.
//!
//! The hold-to-zoom interaction is driven by three fields:
//!
//! * `is_holding` — set when the user presses on the orb, cleared
//!   when they release. Sticky between ticks.
//! * `hold_progress` ∈ `[0, 1]` — integrated each `Tick` using a
//!   real elapsed `dt`. Grows toward `1.0` while held, decays
//!   toward `0.0` while released. Frame-rate independent.
//! * `displayed_speed` / `displayed_zoom` — the canvas inputs
//!   derived from `hold_progress` via `dynamics_for_progress`.
//!
//! Decoupling the boolean intent (`is_holding`) from the integrated
//! progress lets the `Tick` reducer stay pure: every transition
//! produces the same trajectory regardless of how often `Tick`
//! fires, and tests can drive the timeline with synthetic
//! `Instant`s.

use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::Subscription;

use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::ThemeMode;

use crate::application::welcome_dynamics::dynamics_for_progress;
use crate::application::welcome_messages::WelcomeMessage;
use crate::domain::{WelcomeOutcome, WelcomePersistence, WelcomePersistenceError};

pub struct WelcomeState {
    pub theme: OpenSpaceTheme,
    pub theme_mode: ThemeMode,
    pub started_at: Instant,
    pub now: Instant,
    pub persistence: Arc<dyn WelcomePersistence>,
    /// Whether the user is currently holding down the mouse on the
    /// orb. Toggled by `OrbPressed` / `OrbReleased`.
    pub is_holding: bool,
    /// Integrated hold progress in `[0, 1]`. `0.0` is the rest
    /// state; `1.0` is the final form.
    pub hold_progress: f32,
    /// Speed multiplier handed to the canvas, derived from
    /// `hold_progress`.
    pub displayed_speed: f32,
    /// Zoom factor handed to the canvas, derived from
    /// `hold_progress`.
    pub displayed_zoom: f32,
}

impl std::fmt::Debug for WelcomeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WelcomeState")
            .field("theme_mode", &self.theme_mode)
            .field("started_at", &self.started_at)
            .field("now", &self.now)
            .field("is_holding", &self.is_holding)
            .field("hold_progress", &self.hold_progress)
            .field("displayed_speed", &self.displayed_speed)
            .field("displayed_zoom", &self.displayed_zoom)
            .field("persistence", &"<dyn WelcomePersistence>")
            .finish()
    }
}

impl WelcomeState {
    pub fn new(persistence: Arc<dyn WelcomePersistence>, theme_mode: ThemeMode) -> Self {
        let now = Instant::now();
        let (initial_speed, initial_zoom) = dynamics_for_progress(0.0);
        Self {
            theme: OpenSpaceTheme::from_mode(theme_mode),
            theme_mode,
            started_at: now,
            now,
            persistence,
            is_holding: false,
            hold_progress: 0.0,
            displayed_speed: initial_speed,
            displayed_zoom: initial_zoom,
        }
    }

    /// Apply a message and report what the parent should do next.
    ///
    /// Persistence side-effects (`mark_completed`) live in the
    /// router, not here, so unit tests do not need a real filesystem
    /// to drive the welcome flow.
    pub fn update(&mut self, message: WelcomeMessage) -> WelcomeOutcome {
        match message {
            WelcomeMessage::Tick(now) => {
                // Compute dt against the previous tick *before*
                // overwriting `self.now`, so each integration step
                // uses the elapsed wall-clock between ticks rather
                // than assuming a fixed 33ms cadence. Saturating
                // subtraction protects against clock skew.
                let dt = now.saturating_duration_since(self.now).as_secs_f32();
                self.now = now;
                self.advance_orb_progress(dt);
                WelcomeOutcome::None
            }
            WelcomeMessage::ToggleTheme => {
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                self.theme = OpenSpaceTheme::from_mode(self.theme_mode);
                WelcomeOutcome::ThemeToggled(self.theme_mode)
            }
            WelcomeMessage::OrbPressed => {
                self.is_holding = true;
                WelcomeOutcome::None
            }
            WelcomeMessage::OrbReleased => {
                self.is_holding = false;
                WelcomeOutcome::None
            }
            WelcomeMessage::EnterPressed => WelcomeOutcome::Completed,
            WelcomeMessage::Skipped => WelcomeOutcome::Skipped,
        }
    }

    /// Integrate the hold-progress trajectory by `dt` seconds and
    /// re-derive the displayed speed/zoom from it.
    ///
    /// While held, progress ramps toward `1.0` at `HOLD_RAMP_PER_SEC`
    /// (≈1.7s to fill from rest). On release it decays back toward
    /// `0.0` at `RELEASE_RAMP_PER_SEC` (≈1.1s to drain). Linear
    /// integration keeps the math obvious; the perceptual easing is
    /// supplied by `dynamics_for_progress`, which is free to apply
    /// any non-linear curve later without touching this loop.
    fn advance_orb_progress(&mut self, dt: f32) {
        const HOLD_RAMP_PER_SEC: f32 = 0.6;
        const RELEASE_RAMP_PER_SEC: f32 = 0.9;

        let dt = dt.clamp(0.0, 0.25);
        let delta = if self.is_holding {
            HOLD_RAMP_PER_SEC * dt
        } else {
            -RELEASE_RAMP_PER_SEC * dt
        };
        self.hold_progress = (self.hold_progress + delta).clamp(0.0, 1.0);

        let (speed, zoom) = dynamics_for_progress(self.hold_progress);
        self.displayed_speed = speed;
        self.displayed_zoom = zoom;
    }

    /// Subscription driving the orb animation. Targets ~30 fps which
    /// is enough for the slow shimmer in the reference recording and
    /// avoids burning a render-thread core.
    pub fn subscription(&self) -> Subscription<WelcomeMessage> {
        iced::time::every(Duration::from_millis(33)).map(WelcomeMessage::Tick)
    }
}

/// Marks the welcome window as completed in persistence, returning
/// any error so the caller can surface it. Kept as a free function
/// so the router can call it after dispatching the outcome.
pub fn mark_completed(
    persistence: &Arc<dyn WelcomePersistence>,
) -> Result<(), WelcomePersistenceError> {
    persistence.mark_completed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::InMemoryWelcomePersistence;

    #[test]
    fn enter_pressed_yields_completed_outcome() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        let outcome = state.update(WelcomeMessage::EnterPressed);
        assert_eq!(outcome, WelcomeOutcome::Completed);
    }

    #[test]
    fn skipped_yields_skipped_outcome() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        let outcome = state.update(WelcomeMessage::Skipped);
        assert_eq!(outcome, WelcomeOutcome::Skipped);
    }

    #[test]
    fn toggle_theme_swaps_mode() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        let outcome = state.update(WelcomeMessage::ToggleTheme);
        assert_eq!(outcome, WelcomeOutcome::ThemeToggled(ThemeMode::Light));
        assert_eq!(state.theme_mode, ThemeMode::Light);

        let outcome = state.update(WelcomeMessage::ToggleTheme);
        assert_eq!(outcome, WelcomeOutcome::ThemeToggled(ThemeMode::Dark));
    }

    #[test]
    fn tick_updates_now_without_transition() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        let baseline = state.now;
        let later = baseline + Duration::from_millis(120);
        let outcome = state.update(WelcomeMessage::Tick(later));
        assert_eq!(outcome, WelcomeOutcome::None);
        assert!(state.now >= later);
    }

    #[test]
    fn mark_completed_propagates_to_persistence() {
        let store: Arc<dyn WelcomePersistence> = Arc::new(InMemoryWelcomePersistence::new());
        assert!(!store.is_completed());
        super::mark_completed(&store).unwrap();
        assert!(store.is_completed());
    }

    #[test]
    fn fresh_state_starts_at_rest() {
        let state = WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        assert!(!state.is_holding);
        assert_eq!(state.hold_progress, 0.0);
        let (rest_speed, rest_zoom) = dynamics_for_progress(0.0);
        assert!((state.displayed_speed - rest_speed).abs() < 1e-6);
        assert!((state.displayed_zoom - rest_zoom).abs() < 1e-6);
    }

    #[test]
    fn press_then_release_toggles_is_holding_flag() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);

        let outcome = state.update(WelcomeMessage::OrbPressed);
        assert_eq!(outcome, WelcomeOutcome::None);
        assert!(state.is_holding);

        let outcome = state.update(WelcomeMessage::OrbReleased);
        assert_eq!(outcome, WelcomeOutcome::None);
        assert!(!state.is_holding);
    }

    #[test]
    fn holding_for_long_enough_reaches_final_form() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        state.update(WelcomeMessage::OrbPressed);

        // 5 seconds of synthetic ticks at 33ms — comfortably above
        // the natural fill time of ~1.7s so the integration must
        // saturate at 1.0 even in the presence of clamping bugs.
        let mut now = state.now;
        for _ in 0..150 {
            now += Duration::from_millis(33);
            state.update(WelcomeMessage::Tick(now));
        }

        let (max_speed, max_zoom) = dynamics_for_progress(1.0);
        assert!((state.hold_progress - 1.0).abs() < 1e-6);
        assert!((state.displayed_speed - max_speed).abs() < 1e-6);
        assert!((state.displayed_zoom - max_zoom).abs() < 1e-6);
    }

    #[test]
    fn releasing_decays_progress_back_to_rest() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        state.update(WelcomeMessage::OrbPressed);

        // Charge to the final form, then release.
        let mut now = state.now;
        for _ in 0..150 {
            now += Duration::from_millis(33);
            state.update(WelcomeMessage::Tick(now));
        }
        assert!((state.hold_progress - 1.0).abs() < 1e-6);

        state.update(WelcomeMessage::OrbReleased);
        for _ in 0..150 {
            now += Duration::from_millis(33);
            state.update(WelcomeMessage::Tick(now));
        }

        let (rest_speed, rest_zoom) = dynamics_for_progress(0.0);
        assert!(state.hold_progress.abs() < 1e-6);
        assert!((state.displayed_speed - rest_speed).abs() < 1e-6);
        assert!((state.displayed_zoom - rest_zoom).abs() < 1e-6);
    }

    #[test]
    fn release_mid_ramp_decays_without_first_completing() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        state.update(WelcomeMessage::OrbPressed);

        // Hold briefly so we are partway up the ramp but nowhere
        // near the final form.
        let mut now = state.now;
        for _ in 0..15 {
            now += Duration::from_millis(33);
            state.update(WelcomeMessage::Tick(now));
        }
        assert!(state.hold_progress > 0.0);
        assert!(state.hold_progress < 1.0);
        let mid_progress = state.hold_progress;

        // Release mid-ramp — progress should immediately start
        // decaying instead of climbing further.
        state.update(WelcomeMessage::OrbReleased);
        now += Duration::from_millis(33);
        state.update(WelcomeMessage::Tick(now));
        assert!(state.hold_progress < mid_progress);
    }

    #[test]
    fn first_tick_after_press_advances_progress_proportionally_to_dt() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        state.update(WelcomeMessage::OrbPressed);

        let baseline = state.hold_progress;
        let now = state.now + Duration::from_millis(100);
        state.update(WelcomeMessage::Tick(now));

        // 100ms at HOLD_RAMP_PER_SEC=0.6 should add ~0.06 progress.
        let delta = state.hold_progress - baseline;
        assert!(
            (delta - 0.06).abs() < 1e-3,
            "expected ~0.06 progress after 100ms hold, got {delta}"
        );
    }
}
