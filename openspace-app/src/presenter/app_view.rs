//! Top-level view + subscription dispatcher.
//!
//! Each stage owns its own view and subscription. This module
//! folds them into a single Iced surface so `iced::application`
//! sees one source of truth.

use iced::Subscription;
use openspace_home::presenter::app_shell;
use openspace_welcome::presenter::welcome_view as welcome_presenter;

use crate::application::onboarding_app::OnboardingApp;
use crate::domain::app_messages::Message;
use crate::domain::app_stage::Stage;

/// Top-level view dispatcher.
pub fn view(state: &OnboardingApp) -> iced::Element<'_, Message> {
    let stage_element: iced::Element<'_, Message> = match &state.stage {
        Stage::Welcome(welcome) => welcome_presenter::view(welcome).map(Message::Welcome),
        Stage::Home(shell) => app_shell::shell_view(shell).map(Message::Home),
    };

    #[cfg(debug_assertions)]
    {
        let overlay = crate::presenter::app_dev_overlay::view(state);
        iced::widget::stack![stage_element, overlay].into()
    }
    #[cfg(not(debug_assertions))]
    stage_element
}

/// Top-level subscription dispatcher. Each stage owns its own
/// subscription set.
pub fn subscription(state: &OnboardingApp) -> Subscription<Message> {
    let stage_sub = match &state.stage {
        Stage::Welcome(welcome) => welcome.subscription().map(Message::Welcome),
        Stage::Home(shell) => app_shell::shell_subscription(shell).map(Message::Home),
    };

    #[cfg(debug_assertions)]
    {
        // Track window resizes so the dev overlay can show live
        // dimensions regardless of which stage is active.
        let resizes =
            iced::window::resize_events().map(|(_, size)| Message::DevWindowResized(size));
        Subscription::batch([stage_sub, resizes])
    }
    #[cfg(not(debug_assertions))]
    stage_sub
}
