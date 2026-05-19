//! Home feature application layer.
//!
//! Owns:
//!
//! * [`AppRouter`] — applies high-level [`AppCommand`]s to the
//!   session map and surfaces the resulting [`AppEvent`]s.
//! * [`RuntimeManager`] — owns and dispatches to feature
//!   runtimes; activates/deactivates them as sessions change.
//! * [`CommandRegistry`] — merges per-feature command
//!   descriptors and detects shortcut conflicts.
//! * [`filter_by_context_and_query`] — pure filter used by the
//!   palette overlay.
//! * [`new`] — factory used by the welcome → home transition to
//!   build a home shell with a chosen theme mode.

pub mod app_router;
pub mod command_registry;
pub mod home_factory;
pub mod palette_filter;
pub mod runtime_manager;

pub use app_router::AppRouter;
pub use command_registry::CommandRegistry;
pub use home_factory::new;
pub use palette_filter::filter_by_context_and_query;
pub use runtime_manager::RuntimeManager;

// Re-export the feature runtime trait at the application layer for
// convenience; the contract itself lives in the domain layer.
pub use crate::domain::FeatureRuntime;
