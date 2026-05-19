//! Terminal feature value types.
//!
//! Pure data shapes used by the domain trait contracts and the
//! infrastructure adapters. No I/O, no Iced, no async runtime
//! types — those live in the infrastructure layer.

use std::collections::BTreeMap;
use std::path::PathBuf;

use thiserror::Error;

/// Window size sent to the PTY. Mirrors the four-axis shape
/// expected by `portable-pty` so adapters can convert without
/// extra logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtySize {
    pub rows: u16,
    pub cols: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl PtySize {
    pub const fn new(rows: u16, cols: u16) -> Self {
        Self {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

impl Default for PtySize {
    /// Sensible default for unattached PTYs. 80x24 matches the
    /// historical VT100 default and is what most shells assume
    /// when `TIOCGWINSZ` returns nothing useful.
    fn default() -> Self {
        Self::new(24, 80)
    }
}

/// Description of a shell to spawn. Built by the application layer
/// from a [`ResolvedShell`](openspace_platform::ResolvedShell) plus
/// any session-specific overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

impl ShellSpec {
    pub fn new(program: PathBuf) -> Self {
        Self {
            program,
            args: Vec::new(),
            env: BTreeMap::new(),
        }
    }

    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

/// All the failure modes a PTY adapter can report. The variants
/// stay coarse on purpose — feature-level error handling sits in
/// the application layer and only needs to distinguish "spawn
/// failed", "I/O died", and "child gone".
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PtyError {
    #[error("failed to open PTY: {0}")]
    Open(String),
    #[error("failed to spawn shell: {0}")]
    Spawn(String),
    #[error("PTY I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PTY handle already consumed")]
    HandleConsumed,
    #[error("PTY child has exited")]
    ChildExited,
    #[error("PTY resize failed: {0}")]
    Resize(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pty_size_default_is_80x24() {
        let size = PtySize::default();
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
    }

    #[test]
    fn shell_spec_builder_collects_args_and_env() {
        let spec = ShellSpec::new(PathBuf::from("/bin/zsh"))
            .with_arg("-l")
            .with_env("TERM", "xterm-256color");
        assert_eq!(spec.args, vec!["-l".to_string()]);
        assert_eq!(
            spec.env.get("TERM"),
            Some(&"xterm-256color".to_string()),
        );
    }
}
