//! Welcome feature presenter layer.
//!
//! Iced-side rendering: the page view and the canvas program for
//! the centerpiece orb. Presentation-only — no state mutation, no
//! I/O. The host application maps the welcome messages into its
//! own router.

pub mod welcome_orb;
pub mod welcome_view;

pub use welcome_orb::AsciiOrbProgram;
pub use welcome_view::view;
