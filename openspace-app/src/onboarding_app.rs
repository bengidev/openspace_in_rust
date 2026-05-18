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

use iced::Size;
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
// Window sizing
// ---------------------------------------------------------------------------

/// Default window size for the welcome stage. Sized to a compact
/// 4:3 footprint so the welcome window feels like a small dialog
/// rather than a workspace.
pub const WELCOME_DEFAULT_SIZE: Size = Size {
    width: 800.0,
    height: 600.0,
};

/// Minimum window size while the welcome stage is active. Same as
/// the default — the welcome layout is fixed and we do not let the
/// user shrink it further.
pub const WELCOME_MIN_SIZE: Size = Size {
    width: 800.0,
    height: 600.0,
};

/// Default window size for the home stage. Mirrors the original
/// pre-onboarding default.
pub const HOME_DEFAULT_SIZE: Size = Size {
    width: 1280.0,
    height: 800.0,
};

/// Minimum window size while the home stage is active. Tracks the
/// constants in `app_shell` (panels + center + bars).
pub const HOME_MIN_SIZE: Size = Size {
    width: 900.0,
    height: 560.0,
};

// ---------------------------------------------------------------------------
// Messages + state
// ---------------------------------------------------------------------------

/// Top-level message envelope. Sub-stage messages are namespaced via
/// these variants so we can route updates to the correct stage
/// without ambiguity.
#[derive(Debug, Clone)]
pub enum Message {
    /// Window id resolved at startup. We hold onto it so the router
    /// can drive `iced::window::resize` / `set_min_size` /
    /// `set_max_size` when the welcome stage transitions to home.
    Booted(Option<iced::window::Id>),
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
    /// Latest known window id, captured at startup. The router uses
    /// this to drive `iced::window::resize` / `set_min_size` when
    /// the welcome stage finishes and we transition to home.
    pub window_id: Option<iced::window::Id>,
}

impl std::fmt::Debug for OnboardingApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnboardingApp")
            .field("stage", &self.stage)
            .field("window_id", &self.window_id)
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
        Self {
            stage,
            persistence,
            window_id: None,
        }
    }

    /// Initialiser used by `iced::application`. Returns the same
    /// state as [`Self::new`] paired with a startup task that
    /// resolves the window id and applies stage-appropriate sizing
    /// constraints.
    pub fn init(persistence: Arc<dyn WelcomePersistence>) -> (Self, Task<Message>) {
        let state = Self::new(persistence);
        // Fetch the window id; the message handler then applies the
        // initial min-size for whichever stage we booted into.
        let task = iced::window::latest().map(Message::Booted);
        (state, task)
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
    match message {
        Message::Booted(id) => {
            state.window_id = id;
            // Apply stage-appropriate sizing on first boot. The
            // welcome stage runs at a compact default; the home
            // stage uses the larger workspace default.
            apply_stage_window_constraints(state)
        }
        Message::Welcome(msg) => {
            if let Stage::Welcome(welcome) = &mut state.stage {
                let outcome = welcome.update(msg);
                handle_welcome_outcome(state, outcome)
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
    }
}

/// Applies the window sizing rules for the active stage.
///
/// On the welcome stage we clamp the window to a compact pair of
/// dimensions so the layout never grows wider than the four-cell
/// feature row. On the home stage we relax the constraints back to
/// the workspace defaults.
fn apply_stage_window_constraints(state: &OnboardingApp) -> Task<Message> {
    let Some(id) = state.window_id else {
        return Task::none();
    };

    match &state.stage {
        Stage::Welcome(_) => Task::batch([
            iced::window::resize::<Message>(id, WELCOME_DEFAULT_SIZE),
            iced::window::set_min_size::<Message>(id, Some(WELCOME_MIN_SIZE)),
            // Cap the welcome window so the layout cannot grow past
            // the compact reference. Cleared again on transition.
            iced::window::set_max_size::<Message>(id, Some(WELCOME_DEFAULT_SIZE)),
        ]),
        Stage::Home(_) => Task::batch([
            iced::window::set_max_size::<Message>(id, None),
            iced::window::set_min_size::<Message>(id, Some(HOME_MIN_SIZE)),
            iced::window::resize::<Message>(id, HOME_DEFAULT_SIZE),
        ]),
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

            // Relax window sizing back to the home workspace
            // defaults so the user can actually use the app.
            apply_stage_window_constraints(state)
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
    // Start at the welcome size by default. If we boot straight
    // into the home stage (returning user) the `Booted` task will
    // resize up immediately.
    let window = iced::window::Settings {
        size: WELCOME_DEFAULT_SIZE,
        position: iced::window::Position::Centered,
        min_size: Some(WELCOME_MIN_SIZE),
        transparent: true,
        ..iced::window::Settings::default()
    };

    iced::application(
        move || OnboardingApp::init(Arc::clone(&persistence)),
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
