//! Terminal feature domain layer.
//!
//! Pure value types and trait contracts describing terminal state
//! and PTY adapter shape. No real PTY I/O, no Iced concerns.
//! Application and infrastructure depend inward on these types.

pub mod terminal_types;
pub mod pty_adapter;

pub use pty_adapter::{PtyAdapter, PtyHandle, PtyReader, PtyWriter};
pub use terminal_types::{PtyError, PtySize, ShellSpec};
