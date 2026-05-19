//! Window sizing rules for each onboarding stage.
//!
//! The welcome stage runs as a compact dialog so the layout cannot
//! grow past its reference dimensions. The home stage relaxes back
//! to the workspace defaults once the welcome window has been
//! dismissed.

use iced::Size;

/// Default window size for the welcome stage. Sized to a compact
/// 5:4 footprint so the welcome window feels like a small dialog
/// rather than a workspace.
pub const WELCOME_DEFAULT_SIZE: Size = Size {
    width: 1000.0,
    height: 800.0,
};

/// Minimum window size while the welcome stage is active. Same as
/// the default — the welcome layout is fixed and we do not let the
/// user shrink it further.
pub const WELCOME_MIN_SIZE: Size = Size {
    width: 1000.0,
    height: 800.0,
};

/// Default window size for the home stage. Mirrors the original
/// pre-onboarding default.
pub const HOME_DEFAULT_SIZE: Size = Size {
    width: 1280.0,
    height: 800.0,
};

/// Minimum window size while the home stage is active. Tracks the
/// constants in `home_domain::layout` (panels + center + bars).
pub const HOME_MIN_SIZE: Size = Size {
    width: 1280.0,
    height: 800.0,
};
