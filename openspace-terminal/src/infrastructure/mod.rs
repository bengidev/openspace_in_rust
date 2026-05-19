//! Terminal feature infrastructure layer.
//!
//! Concrete adapters: PTY session lifecycle, OS-side process
//! management, output streaming. The application layer depends
//! inward on the domain types and exposes use cases via the
//! [`TerminalCommands`](super::terminal_application::TerminalCommands)
//! provider.

pub mod terminal_pty_adapter;
pub mod terminal_session;

pub use terminal_pty_adapter::{PortablePtyAdapter, PortablePtyHandle};
