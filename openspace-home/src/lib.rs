//! `openspace-home` — post-welcome workspace shell feature crate.
//!
//! The home stage owns the desktop workspace: top bar, side
//! panels, center surface (Terminal / Chat / Editor placeholder
//! views), status bar, command palette overlay, app router
//! orchestrating sessions, the feature runtime manager, and the
//! audit sinks.
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — pure types & contracts: layout constants,
//!   feature runtime trait.
//! * [`application`] — the [`AppRouter`], runtime manager,
//!   command registry, palette + filter logic, and the home-stage
//!   factory used by the welcome → home transition.
//! * [`infrastructure`] — concrete adapters: audit sinks
//!   (no-op + in-memory) and the mock feature runtime used in
//!   tests.
//! * [`presenter`] — Iced-side: app shell view/update,
//!   center-surface placeholders, and the command palette overlay.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;

// Flat re-exports for the host application root.
pub use application::{
    AppRouter, CommandRegistry, FeatureRuntime, RuntimeManager, filter_by_context_and_query, new,
};
pub use domain::layout::{
    HIT_MARGIN, MIN_CENTER_WIDTH, MIN_PANEL_WIDTH, MIN_WINDOW_WIDTH, SEPARATOR_SIZE,
    STATUS_BAR_HEIGHT, TOP_BAR_HEIGHT,
};
pub use infrastructure::{MemoryAuditSink, MockFeatureRuntime, NoopAuditSink};
pub use presenter::{
    AppShell, CommandPaletteOverlay, HomeMessage, PaletteMessage, run, shell_subscription,
    shell_update, shell_view,
};
