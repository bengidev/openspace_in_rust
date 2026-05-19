//! Top-level message envelope for the onboarding router.
//!
//! Sub-stage messages are namespaced via these variants so the
//! router can route updates to the correct stage without
//! ambiguity.

use openspace_home::presenter::HomeMessage;
use openspace_welcome::WelcomeMessage;

#[derive(Debug, Clone)]
pub enum Message {
    /// Window id resolved at startup. We hold onto it so the
    /// router can drive `iced::window::resize` /
    /// `set_min_size` / `set_max_size` when the welcome stage
    /// transitions to home.
    Booted(Option<iced::window::Id>),
    /// Message destined for the welcome stage.
    Welcome(WelcomeMessage),
    /// Message destined for the home stage.
    Home(HomeMessage),
    /// Debug-only: clear the welcome flag and bounce back to the
    /// welcome stage. Used by the dev overlay.
    #[cfg(debug_assertions)]
    DevResetToWelcome,
    /// Debug-only: window resize event used to populate the size
    /// indicator. Carried in its own variant rather than reusing
    /// the home stage's resize handling so the indicator stays
    /// live even when the welcome stage is active.
    #[cfg(debug_assertions)]
    DevWindowResized(iced::Size),
}

impl From<WelcomeMessage> for Message {
    fn from(message: WelcomeMessage) -> Self {
        Message::Welcome(message)
    }
}

impl From<HomeMessage> for Message {
    fn from(message: HomeMessage) -> Self {
        Message::Home(message)
    }
}
