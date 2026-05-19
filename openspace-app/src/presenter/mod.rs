//! Composition-root presenter layer.
//!
//! Top-level Iced view + subscription dispatcher. Routes
//! rendering and event sources to whichever sub-stage is active,
//! and overlays a debug-only chrome (size indicator + "back to
//! welcome" button) on `cfg(debug_assertions)` builds.

pub mod app_view;

#[cfg(debug_assertions)]
pub mod app_dev_overlay;

pub use app_view::{subscription, view};
