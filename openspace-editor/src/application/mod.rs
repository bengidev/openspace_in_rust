//! Editor feature application layer.
//!
//! Owns the rope-based buffer model and the command catalogue
//! published to the home command palette. Real save/dirty
//! orchestration plugs in here as use-cases.

pub mod editor_buffers;
pub mod editor_commands;

pub use editor_commands::EditorCommands;
