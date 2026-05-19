//! Terminal feature application layer.
//!
//! Use cases and orchestration: terminal command catalogue
//! published to the home command palette, plus future per-session
//! orchestration glue between the PTY infrastructure and the
//! terminal presenter.

pub mod terminal_commands;

pub use terminal_commands::TerminalCommands;
