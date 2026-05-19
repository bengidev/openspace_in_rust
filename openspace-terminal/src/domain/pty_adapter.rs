//! PTY adapter contract.
//!
//! The terminal feature talks to the OS through this trait. The
//! real implementation lives in the infrastructure layer and uses
//! `portable-pty`; tests use the mock adapter to drive emulator
//! state transitions deterministically.
//!
//! Keeping the contract in the domain layer means the application
//! layer (session orchestration) and infrastructure layer (real
//! and mock adapters) both depend inward on the same trait.

use std::path::PathBuf;

use super::terminal_types::{PtyError, PtySize, ShellSpec};

/// Owned reader half of a PTY's master end. Consumers run this on
/// a dedicated task and forward bytes into the emulator.
pub trait PtyReader: Send {
    /// Block until at least one byte is available or EOF is hit.
    /// Returns the number of bytes read; `0` means the child closed
    /// its end.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PtyError>;
}

/// Owned writer half of a PTY's master end. Keyboard input flows
/// through here.
pub trait PtyWriter: Send {
    fn write(&mut self, buf: &[u8]) -> Result<usize, PtyError>;
    fn flush(&mut self) -> Result<(), PtyError>;
}

/// Handle to a spawned PTY process. Owns the master fd's reader and
/// writer and the child-process kill switch. Dropping the handle
/// must NOT terminate the child — survival across CenterSurface
/// drops is acceptance criterion #5 on issue #38. Explicit
/// `kill()` is the only termination path.
pub trait PtyHandle: Send {
    /// Take the reader half exactly once. Subsequent calls return
    /// `None`.
    fn take_reader(&mut self) -> Option<Box<dyn PtyReader>>;

    /// Take the writer half exactly once. Subsequent calls return
    /// `None`.
    fn take_writer(&mut self) -> Option<Box<dyn PtyWriter>>;

    /// Resize the PTY window. Sent to the child as `SIGWINCH`.
    fn resize(&mut self, size: PtySize) -> Result<(), PtyError>;

    /// Returns true while the child process is still running.
    fn is_alive(&self) -> bool;

    /// Best-effort termination. Returns once the kill signal has
    /// been delivered; the child may take additional time to exit.
    fn kill(&mut self) -> Result<(), PtyError>;
}

/// Spec describing a PTY spawn request. Built by the application
/// layer from session state and platform policy.
#[derive(Debug, Clone)]
pub struct SpawnRequest {
    pub shell: ShellSpec,
    pub size: PtySize,
    pub cwd: Option<PathBuf>,
}

impl SpawnRequest {
    pub fn new(shell: ShellSpec, size: PtySize) -> Self {
        Self {
            shell,
            size,
            cwd: None,
        }
    }

    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }
}

/// Spawns PTY processes. Concrete adapters live in the
/// infrastructure layer (real `portable-pty`) and tests
/// (mock).
pub trait PtyAdapter: Send + Sync {
    fn spawn(&self, request: SpawnRequest) -> Result<Box<dyn PtyHandle>, PtyError>;
}
