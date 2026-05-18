//! Top-level onboarding router.
//!
//! This module owns the two-stage routing of the desktop app:
//!
//! ```text
//! onboarding-app -> onboarding-welcome -> onboarding-home   // first run
//! onboarding-app -> onboarding-home                         // subsequent runs
//! ```
//!
//! The welcome stage is shown exactly once, gated by a sentinel
//! file under the user's data directory (see
//! [`onboarding_welcome::FileWelcomePersistence`]). On every other
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
//! * `iced::application` is invoked from [`run`]; it accepts an
//!   initialiser closure so we can inject the persistence
//!   implementation. Tests use the in-memory implementation; release
//!   builds use the filesystem-backed one.

use std::sync::Arc;

use iced::Subscription;
use iced::Task;

use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::{BackgroundToken, ForegroundToken, ThemeMode};

use crate::app_shell;
use crate::onboarding_home;
use crate::onboarding_welcome::persistence::{
    FileWelcomePersistence, WelcomePersistence,
};
use crate::onboarding_welcome::welcome::{
    self as welcome_view, WelcomeMessage, WelcomeOutcome, WelcomeState,
};

// ---------------------------------------------------------------------------
// Messages + state
// ---------------------------------------------------------------------------

/// Top-level message envelope. Sub-stage messages are namespaced via
/// these variants so we can route updates to the correct stage
/// without ambiguity.
#[derive(Debug, Clone)]
pub enum Message {
    /// Message destined for the welcome stage.
    Welcome(WelcomeMessage),
    /// Message destined for the home stage.
    Home(app_shell::Message),
}

impl From<WelcomeMessage> for Message {
    fn from(message: WelcomeMessage) -> Self {
        Message::Welcome(message)
    }
}

impl From<app_shell::Message> for Message {
    fn from(message: app_shell::Message) -> Self {
        Message::Home(message)
    }
}

/// Active stage tracked by the router.
///
/// Both variants are boxed so the enum itself stays small. Each
/// stage holds non-trivial state (the welcome state holds animation
/// timestamps and a persistence handle; the home state holds the
/// full app shell with router, theme, command registry, audit
/// sink), so boxing avoids moving large amounts of data on every
/// transition and keeps `OnboardingApp` itself cheap to swap.
pub enum Stage {
    Welcome(Box<WelcomeState>),
    Home(Box<app_shell::AppShell>),
}

impl std::fmt::Debug for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Welcome(_) => f.debug_struct("Stage::Welcome").finish(),
            Stage::Home(_) => f.debug_struct("Stage::Home").finish(),
        }
    }
}

/// Top-level router state. Owns the sub-stage state and the
/// persistence handle.
pub struct OnboardingApp {
    pub stage: Stage,
    pub persistence: Arc<dyn WelcomePersistence>,
}

impl std::fmt::Debug for OnboardingApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnboardingApp")
            .field("stage", &self.stage)
            .field("persistence", &"<dyn WelcomePersistence>")
            .finish()
    }
}

impl OnboardingApp {
    /// Constructs the router with the given persistence backend.
    ///
    /// If the welcome window has already been marked complete on
    /// this device, the router boots straight into the home stage.
    pub fn new(persistence: Arc<dyn WelcomePersistence>) -> Self {
        let stage = if persistence.is_completed() {
            Stage::Home(Box::new(onboarding_home::new(ThemeMode::Dark)))
        } else {
            Stage::Welcome(Box::new(WelcomeState::new(
                Arc::clone(&persistence),
                ThemeMode::Dark,
            )))
        };
        Self { stage, persistence }
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
// Update / view / subscription
// ---------------------------------------------------------------------------

/// Update entry point used by `iced::application`.
///
/// Routes messages to the active sub-stage, then handles any
/// transitions reported by the sub-stage. Persistence side-effects
/// happen here.
pub fn update(state: &mut OnboardingApp, message: Message) -> Task<Message> {
    match (&mut state.stage, message) {
        (Stage::Welcome(welcome), Message::Welcome(msg)) => {
            let outcome = welcome.update(msg);
            handle_welcome_outcome(state, outcome)
        }
        (Stage::Home(shell), Message::Home(msg)) => {
            let task = app_shell::shell_update(shell, msg);
            task.map(Message::Home)
        }
        // Drop messages destined for an inactive stage. This can
        // happen briefly during a transition while in-flight ticks
        // from `iced::time::every` are still being delivered.
        _ => Task::none(),
    }
}

/// Handles the [`WelcomeOutcome`] reported by the welcome stage.
fn handle_welcome_outcome(
    state: &mut OnboardingApp,
    outcome: WelcomeOutcome,
) -> Task<Message> {
    match outcome {
        WelcomeOutcome::None | WelcomeOutcome::ThemeToggled(_) => Task::none(),
        WelcomeOutcome::Completed | WelcomeOutcome::Skipped => {
            let theme_mode = match &state.stage {
                Stage::Welcome(w) => w.theme_mode,
                Stage::Home(s) => s.theme_mode(),
            };

            // Persist the flag. On failure we still transition — the
            // user has clearly indicated intent and re-showing the
            // welcome on next launch would be more annoying than a
            // silent persistence error here. We log it instead.
            if let Err(e) = welcome_view::mark_completed(&state.persistence) {
                tracing::warn!(?e, "failed to persist welcome completion flag");
            }

            state.stage =
                Stage::Home(Box::new(onboarding_home::new(theme_mode)));
            Task::none()
        }
    }
}

/// Top-level view dispatcher.
pub fn view(state: &OnboardingApp) -> iced::Element<'_, Message> {
    match &state.stage {
        Stage::Welcome(welcome) => welcome_view::view(welcome).map(Message::Welcome),
        Stage::Home(shell) => app_shell::shell_view(shell).map(Message::Home),
    }
}

/// Top-level subscription dispatcher. Each stage owns its own
/// subscription set.
pub fn subscription(state: &OnboardingApp) -> Subscription<Message> {
    match &state.stage {
        Stage::Welcome(welcome) => welcome.subscription().map(Message::Welcome),
        Stage::Home(shell) => {
            app_shell::shell_subscription(shell).map(Message::Home)
        }
    }
}

// ---------------------------------------------------------------------------
// Iced entry point
// ---------------------------------------------------------------------------

/// Boot the desktop application using the production
/// (filesystem-backed) persistence.
pub fn run() -> iced::Result {
    let persistence: Arc<dyn WelcomePersistence> = Arc::new(
        FileWelcomePersistence::from_project_dirs().unwrap_or_else(|err| {
            tracing::warn!(
                ?err,
                "could not resolve project data directory; \
                 falling back to in-memory welcome persistence"
            );
            // Fallback so we still launch, even if we cannot
            // persist the welcome flag (sandboxed CI, exotic
            // filesystem layouts, etc.).
            FileWelcomePersistence::new_at(std::env::temp_dir())
        }),
    );
    run_with(persistence)
}

/// Boot the application with an injected persistence backend. Used
/// by integration tests to swap in the in-memory implementation.
pub fn run_with(
    persistence: Arc<dyn WelcomePersistence>,
) -> iced::Result {
    let window = iced::window::Settings {
        size: iced::Size::new(1280.0, 800.0),
        position: iced::window::Position::Centered,
        min_size: Some(iced::Size::new(900.0, 560.0)),
        transparent: true,
        ..iced::window::Settings::default()
    };

    iced::application(
        move || OnboardingApp::new(Arc::clone(&persistence)),
        update,
        view,
    )
    .title("OpenSpace")
    .theme(|state: &OnboardingApp| state.theme().to_iced_theme())
    .style(|state, _theme| iced::theme::Style {
        background_color: state.theme().background(BackgroundToken::Primary),
        text_color: state.theme().foreground(ForegroundToken::Primary),
    })
    .window(window)
    .subscription(subscription)
    .run()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::onboarding_welcome::persistence::InMemoryWelcomePersistence;

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
        let app = new_router(Arc::new(
            InMemoryWelcomePersistence::already_completed(),
        ));
        assert!(matches!(app.stage, Stage::Home(_)));
    }

    #[test]
    fn enter_pressed_marks_persistence_and_transitions_to_home() {
        let store: Arc<dyn WelcomePersistence> =
            Arc::new(InMemoryWelcomePersistence::new());
        let mut app = new_router(Arc::clone(&store));

        let _ = update(
            &mut app,
            Message::Welcome(WelcomeMessage::EnterPressed),
        );

        assert!(matches!(app.stage, Stage::Home(_)));
        assert!(store.is_completed(), "welcome flag must be persisted");
    }

    #[test]
    fn skip_marks_persistence_and_transitions_to_home() {
        let store: Arc<dyn WelcomePersistence> =
            Arc::new(InMemoryWelcomePersistence::new());
        let mut app = new_router(Arc::clone(&store));

        let _ = update(&mut app, Message::Welcome(WelcomeMessage::Skipped));

        assert!(matches!(app.stage, Stage::Home(_)));
        assert!(store.is_completed());
    }

    #[test]
    fn theme_toggle_in_welcome_persists_into_home_after_transition() {
        let store: Arc<dyn WelcomePersistence> =
            Arc::new(InMemoryWelcomePersistence::new());
        let mut app = new_router(Arc::clone(&store));

        // dark by default; toggle to light
        let _ = update(
            &mut app,
            Message::Welcome(WelcomeMessage::ToggleTheme),
        );
        assert_eq!(app.theme_mode(), ThemeMode::Light);

        // accept welcome -> home should keep light mode
        let _ = update(
            &mut app,
            Message::Welcome(WelcomeMessage::EnterPressed),
        );
        assert!(matches!(app.stage, Stage::Home(_)));
        assert_eq!(app.theme_mode(), ThemeMode::Light);
    }

    #[test]
    fn home_messages_after_transition_do_not_panic() {
        let store: Arc<dyn WelcomePersistence> =
            Arc::new(InMemoryWelcomePersistence::already_completed());
        let mut app = new_router(Arc::clone(&store));
        // Sanity: we should be in home and dispatching a Home message
        // should not panic. We use the Home::ToggleTheme path because
        // it is purely local.
        let _ = update(
            &mut app,
            Message::Home(app_shell::Message::ToggleTheme),
        );
    }

    #[test]
    fn welcome_message_after_transition_is_dropped() {
        // Stale ticks from `iced::time::every` may still be delivered
        // after we have transitioned to Home. The router must not
        // panic when that happens.
        let store: Arc<dyn WelcomePersistence> =
            Arc::new(InMemoryWelcomePersistence::already_completed());
        let mut app = new_router(Arc::clone(&store));
        assert!(matches!(app.stage, Stage::Home(_)));

        let now = std::time::Instant::now();
        let _ = update(
            &mut app,
            Message::Welcome(WelcomeMessage::Tick(now)),
        );
        // No panic and stage unchanged.
        assert!(matches!(app.stage, Stage::Home(_)));
    }
}
