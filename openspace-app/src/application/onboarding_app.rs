//! Onboarding router state + reducer.
//!
//! Owns the two-stage routing of the desktop app:
//!
//! ```text
//! onboarding-app -> openspace-welcome -> openspace-home   // first run
//! onboarding-app -> openspace-home                        // subsequent runs
//! ```
//!
//! The welcome stage is shown exactly once, gated by a sentinel
//! file under the user's data directory (see
//! [`openspace_welcome::FileWelcomePersistence`]). On every other
//! launch the router transitions directly into the home shell.
//!
//! Architectural choices:
//!
//! * The router is the single owner of stage transitions. The
//!   welcome state surfaces a [`WelcomeOutcome`]; the home stage
//!   simply runs. Persistence side-effects (`mark_completed`) are
//!   triggered exclusively here so unit tests can drive the welcome
//!   state without touching the filesystem.
//! * The router presents a unified [`Message`] to Iced, lifts
//!   sub-stage messages into variants, and propagates theme toggles
//!   between stages so a toggle on either side carries through to
//!   the other.
//! * `iced::application` is invoked from the infrastructure layer
//!   (`crate::infrastructure::run`); that path accepts an
//!   initialiser closure so we can inject the persistence
//!   implementation. Tests use the in-memory implementation;
//!   release builds use the filesystem-backed one.

use std::sync::Arc;

use iced::Task;

use openspace_home::application::home_factory as home;
use openspace_home::presenter::app_shell;
use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::ThemeMode;
use openspace_welcome::domain::WelcomePersistence;
use openspace_welcome::WelcomeState;

use crate::application::app_router;
use crate::domain::app_messages::Message;
use crate::domain::app_stage::Stage;
use crate::domain::app_window_sizes::WELCOME_DEFAULT_SIZE;

/// Top-level router state. Owns the sub-stage state and the
/// persistence handle.
pub struct OnboardingApp {
    pub stage: Stage,
    pub persistence: Arc<dyn WelcomePersistence>,
    /// Latest known window id, captured at startup. The router
    /// uses this to drive `iced::window::resize` /
    /// `set_min_size` when the welcome stage finishes and we
    /// transition to home.
    pub window_id: Option<iced::window::Id>,
    /// Latest known window size. Maintained from the resize event
    /// stream and surfaced in the debug overlay.
    #[cfg(debug_assertions)]
    pub window_size: iced::Size,
}

impl std::fmt::Debug for OnboardingApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("OnboardingApp");
        s.field("stage", &self.stage)
            .field("window_id", &self.window_id);
        #[cfg(debug_assertions)]
        s.field("window_size", &self.window_size);
        s.field("persistence", &"<dyn WelcomePersistence>").finish()
    }
}

impl OnboardingApp {
    /// Constructs the router with the given persistence backend.
    ///
    /// If the welcome window has already been marked complete on
    /// this device, the router boots straight into the home stage.
    pub fn new(persistence: Arc<dyn WelcomePersistence>) -> Self {
        let stage = if persistence.is_completed() {
            Stage::Home(Box::new(home::new(ThemeMode::Dark)))
        } else {
            Stage::Welcome(Box::new(WelcomeState::new(
                Arc::clone(&persistence),
                ThemeMode::Dark,
            )))
        };
        Self {
            stage,
            persistence,
            window_id: None,
            #[cfg(debug_assertions)]
            window_size: WELCOME_DEFAULT_SIZE,
        }
    }

    /// Returns the active theme mode regardless of stage so the
    /// chrome layer can pick a consistent style.
    pub fn theme_mode(&self) -> ThemeMode {
        match &self.stage {
            Stage::Welcome(welcome) => welcome.theme_mode,
            Stage::Home(shell) => shell.theme_mode(),
        }
    }

    /// Returns the resolved theme matching the active stage.
    pub fn theme(&self) -> OpenSpaceTheme {
        OpenSpaceTheme::from_mode(self.theme_mode())
    }
}

// ---------------------------------------------------------------------------
// Initialiser
// ---------------------------------------------------------------------------

/// Initialiser used by `iced::application`. Returns the same state
/// as [`OnboardingApp::new`] paired with a startup task that
/// resolves the window id and applies stage-appropriate sizing
/// constraints.
pub fn init(persistence: Arc<dyn WelcomePersistence>) -> (OnboardingApp, Task<Message>) {
    let state = OnboardingApp::new(persistence);
    // Fetch the window id; the message handler then applies the
    // initial min-size for whichever stage we booted into.
    let task = iced::window::latest().map(Message::Booted);
    (state, task)
}

// ---------------------------------------------------------------------------
// Update reducer
// ---------------------------------------------------------------------------

/// Update entry point used by `iced::application`.
///
/// Routes messages to the active sub-stage, then handles any
/// transitions reported by the sub-stage. Persistence side-effects
/// happen via [`crate::application::app_router`].
pub fn update(state: &mut OnboardingApp, message: Message) -> Task<Message> {
    match message {
        Message::Booted(id) => {
            state.window_id = id;
            // Apply stage-appropriate sizing on first boot. The
            // welcome stage runs at a compact default; the home
            // stage uses the larger workspace default.
            app_router::apply_stage_window_constraints(state)
        }
        Message::Welcome(msg) => {
            if let Stage::Welcome(welcome) = &mut state.stage {
                let outcome = welcome.update(msg);
                app_router::handle_welcome_outcome(state, outcome)
            } else {
                // Stale tick from `iced::time::every` after the
                // stage transitioned. Drop it.
                Task::none()
            }
        }
        Message::Home(msg) => {
            if let Stage::Home(shell) = &mut state.stage {
                let task = app_shell::shell_update(shell, msg);
                task.map(Message::Home)
            } else {
                Task::none()
            }
        }
        #[cfg(debug_assertions)]
        Message::DevResetToWelcome => app_router::handle_dev_reset(state),
        #[cfg(debug_assertions)]
        Message::DevWindowResized(size) => {
            state.window_size = size;
            Task::none()
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use openspace_welcome::application::welcome_messages::WelcomeMessage;
    use openspace_welcome::infrastructure::InMemoryWelcomePersistence;

    fn new_router(persistence: Arc<dyn WelcomePersistence>) -> OnboardingApp {
        OnboardingApp::new(persistence)
    }

    #[test]
    fn fresh_install_starts_in_welcome_stage() {
        let app = new_router(Arc::new(InMemoryWelcomePersistence::new()));
        assert!(matches!(app.stage, Stage::Welcome(_)));
    }

    #[test]
    fn returning_user_starts_in_home_stage() {
        let app = new_router(Arc::new(InMemoryWelcomePersistence::already_completed()));
        assert!(matches!(app.stage, Stage::Home(_)));
    }

    #[test]
    fn enter_pressed_marks_persistence_and_transitions_to_home() {
        let store: Arc<dyn WelcomePersistence> = Arc::new(InMemoryWelcomePersistence::new());
        let mut app = new_router(Arc::clone(&store));

        let _ = update(&mut app, Message::Welcome(WelcomeMessage::EnterPressed));

        assert!(matches!(app.stage, Stage::Home(_)));
        assert!(store.is_completed(), "welcome flag must be persisted");
    }

    #[test]
    fn skip_marks_persistence_and_transitions_to_home() {
        let store: Arc<dyn WelcomePersistence> = Arc::new(InMemoryWelcomePersistence::new());
        let mut app = new_router(Arc::clone(&store));

        let _ = update(&mut app, Message::Welcome(WelcomeMessage::Skipped));

        assert!(matches!(app.stage, Stage::Home(_)));
        assert!(store.is_completed());
    }

    #[test]
    fn theme_toggle_in_welcome_persists_into_home_after_transition() {
        let store: Arc<dyn WelcomePersistence> = Arc::new(InMemoryWelcomePersistence::new());
        let mut app = new_router(Arc::clone(&store));

        // dark by default; toggle to light
        let _ = update(&mut app, Message::Welcome(WelcomeMessage::ToggleTheme));
        assert_eq!(app.theme_mode(), ThemeMode::Light);

        // accept welcome -> home should keep light mode
        let _ = update(&mut app, Message::Welcome(WelcomeMessage::EnterPressed));
        assert!(matches!(app.stage, Stage::Home(_)));
        assert_eq!(app.theme_mode(), ThemeMode::Light);
    }

    #[test]
    fn home_messages_after_transition_do_not_panic() {
        let store: Arc<dyn WelcomePersistence> =
            Arc::new(InMemoryWelcomePersistence::already_completed());
        let mut app = new_router(Arc::clone(&store));
        // Sanity: we should be in home and dispatching a Home
        // message should not panic. We use the Home::ToggleTheme
        // path because it is purely local.
        let _ = update(
            &mut app,
            Message::Home(openspace_home::presenter::HomeMessage::ToggleTheme),
        );
    }

    #[test]
    fn welcome_message_after_transition_is_dropped() {
        // Stale ticks from `iced::time::every` may still be
        // delivered after we have transitioned to Home. The router
        // must not panic when that happens.
        let store: Arc<dyn WelcomePersistence> =
            Arc::new(InMemoryWelcomePersistence::already_completed());
        let mut app = new_router(Arc::clone(&store));
        assert!(matches!(app.stage, Stage::Home(_)));

        let now = std::time::Instant::now();
        let _ = update(&mut app, Message::Welcome(WelcomeMessage::Tick(now)));
        // No panic and stage unchanged.
        assert!(matches!(app.stage, Stage::Home(_)));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn dev_reset_clears_persistence_and_returns_to_welcome() {
        let store: Arc<dyn WelcomePersistence> =
            Arc::new(InMemoryWelcomePersistence::already_completed());
        let mut app = new_router(Arc::clone(&store));
        assert!(matches!(app.stage, Stage::Home(_)));

        let _ = update(&mut app, Message::DevResetToWelcome);

        assert!(matches!(app.stage, Stage::Welcome(_)));
        assert!(
            !store.is_completed(),
            "dev reset must clear the welcome flag"
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    fn dev_window_resized_updates_tracked_size() {
        let store: Arc<dyn WelcomePersistence> = Arc::new(InMemoryWelcomePersistence::new());
        let mut app = new_router(store);
        let new_size = iced::Size::new(1234.0, 567.0);
        let _ = update(&mut app, Message::DevWindowResized(new_size));
        assert_eq!(app.window_size, new_size);
    }
}
