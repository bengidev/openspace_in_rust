//! Composition-root infrastructure layer.
//!
//! Concrete bootstrap for the desktop application: wires the
//! filesystem-backed welcome persistence into the Iced runtime
//! and surfaces a `run_with` entry point so integration tests can
//! swap in an in-memory persistence implementation.

pub mod app_bootstrap;

pub use app_bootstrap::{run, run_with};
