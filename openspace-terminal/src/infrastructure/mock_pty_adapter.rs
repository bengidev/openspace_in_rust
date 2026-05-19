//! In-memory mock PTY adapter for deterministic tests.
//!
//! The mock satisfies the [`PtyAdapter`] / [`PtyHandle`] contracts
//! without touching the OS. Tests preload bytes the read half should
//! return (one chunk per `read()` call), capture every byte the
//! write half accepts, and toggle the alive flag explicitly to drive
//! emulator state transitions.
//!
//! Acceptance criterion #6 on issue #38: a mock PTY adapter must
//! exist for deterministic integration tests of emulator state
//! transitions.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::domain::pty_adapter::{PtyAdapter, PtyHandle, PtyReader, PtyWriter, SpawnRequest};
use crate::domain::terminal_types::{PtyError, PtySize};

#[derive(Debug, Default, Clone)]
pub struct MockPtyState {
    pub spawn_count: usize,
    pub last_request: Option<SpawnRequestSnapshot>,
    pub written: Vec<u8>,
    pub kill_count: usize,
    pub resize_count: usize,
    pub last_size: Option<PtySize>,
    pub alive: bool,
    pub flushed: usize,
}

/// Plain snapshot of a [`SpawnRequest`] for assertion. Avoids
/// pinning tests to the lifetime of the original request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnRequestSnapshot {
    pub program: std::path::PathBuf,
    pub args: Vec<String>,
    pub size: PtySize,
}

impl SpawnRequestSnapshot {
    fn from_request(r: &SpawnRequest) -> Self {
        Self {
            program: r.shell.program.clone(),
            args: r.shell.args.clone(),
            size: r.size,
        }
    }
}

/// Shared inner state for a single mock PTY. Multiple ends of the
/// PTY (reader, writer, handle) hand out clones of this `Arc` so
/// tests inspect everything from one place.
#[derive(Debug, Default)]
struct MockInner {
    read_chunks: VecDeque<Vec<u8>>,
    state: MockPtyState,
}

#[derive(Debug, Clone, Default)]
pub struct MockPtyAdapter {
    inner: Arc<Mutex<MockInner>>,
}

impl MockPtyAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue bytes that the next reads should return. Each push
    /// becomes one `read()` chunk, in FIFO order. Once the queue
    /// drains, `read()` returns 0 (EOF semantics).
    pub fn push_read_chunk(&self, bytes: impl Into<Vec<u8>>) {
        self.inner
            .lock()
            .unwrap()
            .read_chunks
            .push_back(bytes.into());
    }

    /// Snapshot of everything tests need to assert on.
    pub fn state(&self) -> MockPtyState {
        self.inner.lock().unwrap().state.clone()
    }

    /// Mark the child as exited; subsequent `is_alive()` calls
    /// return false.
    pub fn mark_exited(&self) {
        self.inner.lock().unwrap().state.alive = false;
    }
}

impl PtyAdapter for MockPtyAdapter {
    fn spawn(&self, request: SpawnRequest) -> Result<Box<dyn PtyHandle>, PtyError> {
        let mut inner = self.inner.lock().unwrap();
        inner.state.spawn_count += 1;
        inner.state.last_request = Some(SpawnRequestSnapshot::from_request(&request));
        inner.state.alive = true;
        drop(inner);

        Ok(Box::new(MockPtyHandle {
            inner: self.inner.clone(),
            reader: Some(Box::new(MockPtyReader {
                inner: self.inner.clone(),
            })),
            writer: Some(Box::new(MockPtyWriter {
                inner: self.inner.clone(),
            })),
        }))
    }
}

struct MockPtyReader {
    inner: Arc<Mutex<MockInner>>,
}

impl PtyReader for MockPtyReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PtyError> {
        let mut guard = self.inner.lock().unwrap();
        match guard.read_chunks.pop_front() {
            Some(chunk) => {
                let n = chunk.len().min(buf.len());
                buf[..n].copy_from_slice(&chunk[..n]);
                if n < chunk.len() {
                    // Push the unread tail back so the next call
                    // continues where this one left off.
                    let tail = chunk[n..].to_vec();
                    guard.read_chunks.push_front(tail);
                }
                Ok(n)
            }
            None => Ok(0),
        }
    }
}

struct MockPtyWriter {
    inner: Arc<Mutex<MockInner>>,
}

impl PtyWriter for MockPtyWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, PtyError> {
        let mut guard = self.inner.lock().unwrap();
        guard.state.written.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), PtyError> {
        let mut guard = self.inner.lock().unwrap();
        guard.state.flushed += 1;
        Ok(())
    }
}

pub struct MockPtyHandle {
    inner: Arc<Mutex<MockInner>>,
    reader: Option<Box<dyn PtyReader>>,
    writer: Option<Box<dyn PtyWriter>>,
}

impl PtyHandle for MockPtyHandle {
    fn take_reader(&mut self) -> Option<Box<dyn PtyReader>> {
        self.reader.take()
    }

    fn take_writer(&mut self) -> Option<Box<dyn PtyWriter>> {
        self.writer.take()
    }

    fn resize(&mut self, size: PtySize) -> Result<(), PtyError> {
        let mut guard = self.inner.lock().unwrap();
        guard.state.resize_count += 1;
        guard.state.last_size = Some(size);
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.inner.lock().unwrap().state.alive
    }

    fn kill(&mut self) -> Result<(), PtyError> {
        let mut guard = self.inner.lock().unwrap();
        guard.state.kill_count += 1;
        guard.state.alive = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::domain::terminal_types::ShellSpec;

    fn fixture_request() -> SpawnRequest {
        SpawnRequest::new(
            ShellSpec::new(PathBuf::from("/bin/zsh")).with_arg("-l"),
            PtySize::new(24, 80),
        )
    }

    #[test]
    fn spawn_records_request_and_marks_alive() {
        let adapter = MockPtyAdapter::new();
        let _handle = adapter.spawn(fixture_request()).unwrap();
        let state = adapter.state();
        assert_eq!(state.spawn_count, 1);
        assert!(state.alive);
        let snap = state.last_request.expect("snapshot recorded");
        assert_eq!(snap.program, PathBuf::from("/bin/zsh"));
        assert_eq!(snap.args, vec!["-l".to_string()]);
    }

    #[test]
    fn read_returns_queued_chunks_then_eof() {
        let adapter = MockPtyAdapter::new();
        adapter.push_read_chunk(b"hello".to_vec());
        let mut handle = adapter.spawn(fixture_request()).unwrap();
        let mut reader = handle.take_reader().unwrap();
        let mut buf = [0u8; 16];
        let n = reader.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"hello");
        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn write_captures_bytes_and_flush_increments_counter() {
        let adapter = MockPtyAdapter::new();
        let mut handle = adapter.spawn(fixture_request()).unwrap();
        let mut writer = handle.take_writer().unwrap();
        writer.write(b"ls -la\n").unwrap();
        writer.flush().unwrap();
        let state = adapter.state();
        assert_eq!(state.written, b"ls -la\n");
        assert_eq!(state.flushed, 1);
    }

    #[test]
    fn kill_marks_dead_and_increments_counter() {
        let adapter = MockPtyAdapter::new();
        let mut handle = adapter.spawn(fixture_request()).unwrap();
        assert!(handle.is_alive());
        handle.kill().unwrap();
        assert!(!handle.is_alive());
        assert_eq!(adapter.state().kill_count, 1);
    }
}
