//! Layout constants for the home shell.
//!
//! Captures the geometry rules of the workspace shell so the view,
//! the update reducer, and any cross-cutting code (window size
//! enforcement, hit testing) all read from the same numbers.

/// Top bar height in logical pixels.
pub const TOP_BAR_HEIGHT: f32 = 48.0;

/// Status bar height in logical pixels.
pub const STATUS_BAR_HEIGHT: f32 = 28.0;

/// Visual width of every separator drawn between regions.
pub const SEPARATOR_SIZE: f32 = 1.0;

/// Minimum width allowed for either side panel.
pub const MIN_PANEL_WIDTH: f32 = 100.0;

/// Minimum width allowed for the center surface so its content can
/// always render.
pub const MIN_CENTER_WIDTH: f32 = 200.0;

/// Generous hit margin for the resize handles so the user does not
/// have to land directly on the 1px separator.
pub const HIT_MARGIN: f32 = 3.0;

/// Minimum total window width so the layout does not collapse.
/// 2 panels + center + 2 separators.
pub const MIN_WINDOW_WIDTH: f32 = MIN_PANEL_WIDTH * 2.0 + MIN_CENTER_WIDTH + 2.0 * SEPARATOR_SIZE;
