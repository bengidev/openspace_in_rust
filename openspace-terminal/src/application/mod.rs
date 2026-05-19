//! Terminal feature application layer.
//!
//! Use cases and orchestration: terminal command catalogue
//! published to the home command palette, plus future per-session
//! orchestration glue between the PTY infrastructure and the
//! terminal presenter.

pub mod terminal_commands;
pub mod terminal_emulator;
pub mod terminal_io_loop;
pub mod terminal_spawn;

pub use terminal_commands::TerminalCommands;
pub use terminal_emulator::Emulator;
pub use terminal_io_loop::{
    DEFAULT_CHANNEL_CAPACITY, DEFAULT_READ_CHUNK, ReadLoop, ReadLoopOutcome, WriteLoop,
    WriteLoopOutcome, spawn_read_loop, spawn_read_loop_with, spawn_write_loop,
    spawn_write_loop_with,
};
pub use terminal_spawn::{SAFE_ENV_KEYS, build_spawn_request, build_spawn_request_with};
