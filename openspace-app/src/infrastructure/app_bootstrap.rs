//! Iced bootstrap entry point.
//!
//! `run` is the production entry point used by `main`; it loads
//! the filesystem-backed welcome persistence and falls back to a
//! temp-dir backed instance if the user data directory cannot be
//! resolved (sandboxed CI, exotic filesystem layouts, etc.).
//!
//! `run_with` accepts an injected persistence backend so
//! integration tests can drive the welcome flow without touching
//! disk.

use std::sync::Arc;

use openspace_theme::tokens::{BackgroundToken, ForegroundToken};
use openspace_welcome::domain::WelcomePersistence;
use openspace_welcome::infrastructure::FileWelcomePersistence;

use crate::application::onboarding_app::{self, OnboardingApp};
use crate::domain::app_window_sizes::{WELCOME_DEFAULT_SIZE, WELCOME_MIN_SIZE};
use crate::presenter::{subscription, view};

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

/// Boot the application with an injected persistence backend.
/// Used by integration tests to swap in the in-memory
/// implementation.
pub fn run_with(persistence: Arc<dyn WelcomePersistence>) -> iced::Result {
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
        move || onboarding_app::init(Arc::clone(&persistence)),
        onboarding_app::update,
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
