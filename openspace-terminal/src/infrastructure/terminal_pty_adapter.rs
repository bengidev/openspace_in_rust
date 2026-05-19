//! Real PTY adapter backed by `portable-pty`.
//!
//! The adapter spawns a child shell on a freshly opened PTY,
//! drops the slave fd so EOF semantics stay clean, and hands back
//! a [`PortablePtyHandle`] that owns the master end and the
//! child-process kill switch.
//!
//! Crucially, dropping the handle does NOT terminate the child.
//! The handle's lifetime is independent of the CenterSurface view
//! — that's what acceptance criterion #5 on issue #38 (mode switch
//! survival) requires. Termination is explicit via [`PtyHandle::kill`].

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize as RawSize, native_pty_system};

use crate::domain::pty_adapter::{PtyAdapter, PtyHandle, PtyReader, PtyWriter, SpawnRequest};
use crate::domain::terminal_types::{PtyError, PtySize};

pub struct PortablePtyAdapter;

impl PortablePtyAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PortablePtyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

fn raw_size(size: PtySize) -> RawSize {
    RawSize {
        rows: size.rows,
        cols: size.cols,
        pixel_width: size.pixel_width,
        pixel_height: size.pixel_height,
    }
}

fn build_command(request: &SpawnRequest) -> CommandBuilder {
    let mut cmd = CommandBuilder::new(&request.shell.program);
    for arg in &request.shell.args {
        cmd.arg(arg);
    }
    // Acceptance criterion #2: spawn with the user's environment
    // sans secrets. The application layer is responsible for
    // assembling a safe env in ShellSpec.env; we clear inherited
    // env so nothing leaks through accidentally.
    cmd.env_clear();
    for (k, v) in &request.shell.env {
        cmd.env(k, v);
    }
    if let Some(cwd) = &request.cwd {
        cmd.cwd(cwd);
    }
    cmd
}

impl PtyAdapter for PortablePtyAdapter {
    fn spawn(&self, request: SpawnRequest) -> Result<Box<dyn PtyHandle>, PtyError> {
        let system = native_pty_system();
        let pair = system
            .openpty(raw_size(request.size))
            .map_err(|e| PtyError::Open(e.to_string()))?;

        let cmd = build_command(&request);
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::Spawn(e.to_string()))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::Open(format!("clone reader: {e}")))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::Open(format!("take writer: {e}")))?;

        // Drop the slave fd so the child sees EOF as soon as the
        // master goes away. Keeps read-loop exit semantics honest.
        drop(pair.slave);

        Ok(Box::new(PortablePtyHandle::new(
            pair.master,
            child,
            reader,
            writer,
        )))
    }
}

/// Reader half wrapping a blocking `Read + Send` from portable-pty.
pub struct PortablePtyReader {
    inner: Box<dyn Read + Send>,
}

impl PortablePtyReader {
    fn new(inner: Box<dyn Read + Send>) -> Self {
        Self { inner }
    }
}

impl PtyReader for PortablePtyReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PtyError> {
        Ok(self.inner.read(buf)?)
    }
}

/// Writer half wrapping a blocking `Write + Send` from portable-pty.
pub struct PortablePtyWriter {
    inner: Box<dyn Write + Send>,
}

impl PortablePtyWriter {
    fn new(inner: Box<dyn Write + Send>) -> Self {
        Self { inner }
    }
}

impl PtyWriter for PortablePtyWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, PtyError> {
        Ok(self.inner.write(buf)?)
    }

    fn flush(&mut self) -> Result<(), PtyError> {
        Ok(self.inner.flush()?)
    }
}

/// Owned PTY handle. The master fd and child process live behind
/// `Arc<Mutex<_>>` so the handle can be moved between owners (e.g.
/// re-attached to a new CenterSurface) without taking the child
/// down with it. Termination is explicit via [`Self::kill`].
pub struct PortablePtyHandle {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
    reader: Option<Box<dyn PtyReader>>,
    writer: Option<Box<dyn PtyWriter>>,
}

impl PortablePtyHandle {
    fn new(
        master: Box<dyn MasterPty + Send>,
        child: Box<dyn Child + Send + Sync>,
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
    ) -> Self {
        Self {
            master: Arc::new(Mutex::new(master)),
            child: Arc::new(Mutex::new(child)),
            reader: Some(Box::new(PortablePtyReader::new(reader))),
            writer: Some(Box::new(PortablePtyWriter::new(writer))),
        }
    }
}

impl PtyHandle for PortablePtyHandle {
    fn take_reader(&mut self) -> Option<Box<dyn PtyReader>> {
        self.reader.take()
    }

    fn take_writer(&mut self) -> Option<Box<dyn PtyWriter>> {
        self.writer.take()
    }

    fn resize(&mut self, size: PtySize) -> Result<(), PtyError> {
        self.master
            .lock()
            .unwrap()
            .resize(raw_size(size))
            .map_err(|e| PtyError::Resize(e.to_string()))
    }

    fn is_alive(&self) -> bool {
        let mut guard = self.child.lock().unwrap();
        matches!(guard.try_wait(), Ok(None))
    }

    fn kill(&mut self) -> Result<(), PtyError> {
        self.child
            .lock()
            .unwrap()
            .kill()
            .map_err(|e| PtyError::Spawn(format!("kill: {e}")))
    }
}
