//! Stage-routing helpers used by the onboarding router.
//!
//! Pulled into their own module so the [`OnboardingApp`] reducer
//! reads as a flat dispatch table while the per-outcome /
//! per-window-event behaviour lives in named functions.

use std::sync::Arc;

use iced::Task;

use openspace_home::application::home_factory as home;
use openspace_welcome::application::welcome_state;
use openspace_welcome::domain::WelcomeOutcome;
use openspace_welcome::WelcomeState;

use crate::application::onboarding_app::OnboardingApp;
use crate::domain::app_messages::Message;
use crate::domain::app_stage::Stage;
use crate::domain::app_window_sizes::{
    HOME_DEFAULT_SIZE, HOME_MIN_SIZE, WELCOME_DEFAULT_SIZE, WELCOME_MIN_SIZE,
};

/// Applies the window sizing rules for the active stage.
///
/// On the welcome stage we clamp the window to a compact pair of
/// dimensions so the layout never grows wider than the four-cell
/// feature row. On the home stage we relax the constraints back to
/// the workspace defaults.
pub fn apply_stage_window_constraints(state: &OnboardingApp) -> Task<Message> {
    let Some(id) = state.window_id else {
        return Task::none();
    };

    match &state.stage {
        Stage::Welcome(_) => Task::batch([
            iced::window::resize::<Message>(id, WELCOME_DEFAULT_SIZE),
            iced::window::set_min_size::<Message>(id, Some(WELCOME_MIN_SIZE)),
            // Cap the welcome window so the layout cannot grow
            // past the compact reference. Cleared again on
            // transition.
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
pub fn handle_welcome_outcome(state: &mut OnboardingApp, outcome: WelcomeOutcome) -> Task<Message> {
    match outcome {
        WelcomeOutcome::None | WelcomeOutcome::ThemeToggled(_) => Task::none(),
        WelcomeOutcome::Completed | WelcomeOutcome::Skipped => {
            let theme_mode = match &state.stage {
                Stage::Welcome(w) => w.theme_mode,
                Stage::Home(s) => s.theme_mode(),
            };

            // Persist the flag. On failure we still transition —
            // the user has clearly indicated intent and re-showing
            // the welcome on next launch would be more annoying
            // than a silent persistence error here. We log it
            // instead.
            if let Err(e) = welcome_state::mark_completed(&state.persistence) {
                tracing::warn!(?e, "failed to persist welcome completion flag");
            }

            state.stage = Stage::Home(Box::new(home::new(theme_mode)));

            // Relax window sizing back to the home workspace
            // defaults so the user can actually use the app.
            apply_stage_window_constraints(state)
        }
    }
}

/// Debug-only handler for the "back to welcome" action. Clears the
/// persistence flag, swaps the stage back to Welcome, and re-applies
/// the welcome-stage window constraints.
#[cfg(debug_assertions)]
pub fn handle_dev_reset(state: &mut OnboardingApp) -> Task<Message> {
    if let Err(e) = state.persistence.reset() {
        tracing::warn!(?e, "dev: failed to reset welcome flag");
    }
    let theme_mode = match &state.stage {
        Stage::Welcome(w) => w.theme_mode,
        Stage::Home(s) => s.theme_mode(),
    };
    state.stage = Stage::Welcome(Box::new(WelcomeState::new(
        Arc::clone(&state.persistence),
        theme_mode,
    )));
    apply_stage_window_constraints(state)
}
