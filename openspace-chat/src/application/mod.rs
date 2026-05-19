//! Chat feature application layer.
//!
//! Use cases and orchestration: chat thread lifecycle, channel
//! routing, plus the chat command catalogue published to the home
//! command palette.

pub mod chat_channels;
pub mod chat_commands;

pub use chat_commands::ChatCommands;
