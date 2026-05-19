//! Async I/O loops bridging the blocking [`PtyReader`] / [`PtyWriter`]
//! contracts to tokio-friendly channels.
//!
//! The PTY adapter speaks blocking I/O — that's what `portable-pty`
//! gives us and what the mock adapter mirrors. The rest of the
//! terminal feature wants async streams. These helpers run the
//! blocking halves on dedicated `spawn_blocking` tasks and surface
//! mpsc channels for the application layer to consume.
//!
//! Acceptance criteria #3 and #4 on issue #38: async read loop
//! streaming PTY stdout, async write loop sending keyboard input.

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::domain::pty_adapter::{PtyReader, PtyWriter};
use crate::domain::terminal_types::PtyError;

/// Default chunk size handed to `PtyReader::read`. Big enough to
/// soak up bursts of escape sequences from a real shell, small
/// enough that one stuck reader does not eat memory.
pub const DEFAULT_READ_CHUNK: usize = 4096;

/// Default channel capacity. Keeps a couple of read chunks buffered
/// without unbounded growth if the consumer falls behind.
pub const DEFAULT_CHANNEL_CAPACITY: usize = 32;

/// Outcome of the read loop. Lets the application layer
/// distinguish a clean child exit from a transport error.
#[derive(Debug)]
pub enum ReadLoopOutcome {
    /// `read()` returned 0; the child closed its end.
    Eof,
    /// The reader bubbled up an error.
    Error(PtyError),
}

/// Outcome of the write loop. Mirrors the read side.
#[derive(Debug)]
pub enum WriteLoopOutcome {
    /// The input channel was closed; nothing left to write.
    InputClosed,
    /// The writer bubbled up an error.
    Error(PtyError),
}

/// Handle to the spawned read loop. Holding it keeps the receiver
/// alive; dropping it lets the loop exit on its next read attempt
/// once the receiver hangs up.
pub struct ReadLoop {
    pub bytes: mpsc::Receiver<Vec<u8>>,
    pub task: JoinHandle<ReadLoopOutcome>,
}

/// Handle to the spawned write loop. The sender feeds bytes to the
/// PTY's stdin; closing it causes the loop to exit cleanly.
pub struct WriteLoop {
    pub input: mpsc::Sender<Vec<u8>>,
    pub task: JoinHandle<WriteLoopOutcome>,
}

/// Spawn the async read loop over a blocking [`PtyReader`].
///
/// Reads happen on a `spawn_blocking` worker so the tokio runtime
/// stays responsive while the PTY blocks on syscalls. Each read
/// chunk is sent through the returned channel; a zero-byte read
/// (EOF) ends the loop with [`ReadLoopOutcome::Eof`].
pub fn spawn_read_loop(reader: Box<dyn PtyReader>) -> ReadLoop {
    spawn_read_loop_with(reader, DEFAULT_READ_CHUNK, DEFAULT_CHANNEL_CAPACITY)
}

/// Variant exposing chunk size and channel capacity for tests and
/// tuning.
pub fn spawn_read_loop_with(
    mut reader: Box<dyn PtyReader>,
    chunk_size: usize,
    channel_capacity: usize,
) -> ReadLoop {
    let (tx, rx) = mpsc::channel::<Vec<u8>>(channel_capacity);

    let task = tokio::task::spawn_blocking(move || {
        let mut buf = vec![0u8; chunk_size];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => return ReadLoopOutcome::Eof,
                Ok(n) => {
                    let chunk = buf[..n].to_vec();
                    if tx.blocking_send(chunk).is_err() {
                        // Consumer dropped the receiver. Exit
                        // cleanly so the worker thread is freed.
                        return ReadLoopOutcome::Eof;
                    }
                }
                Err(e) => return ReadLoopOutcome::Error(e),
            }
        }
    });

    ReadLoop { bytes: rx, task }
}

/// Spawn the async write loop over a blocking [`PtyWriter`].
///
/// Bytes pushed into the returned sender are written to the PTY in
/// FIFO order. Closing the sender (drop) terminates the loop with
/// [`WriteLoopOutcome::InputClosed`].
pub fn spawn_write_loop(writer: Box<dyn PtyWriter>) -> WriteLoop {
    spawn_write_loop_with(writer, DEFAULT_CHANNEL_CAPACITY)
}

/// Variant exposing channel capacity for tests and tuning.
pub fn spawn_write_loop_with(
    mut writer: Box<dyn PtyWriter>,
    channel_capacity: usize,
) -> WriteLoop {
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(channel_capacity);

    let task = tokio::task::spawn_blocking(move || {
        loop {
            match rx.blocking_recv() {
                Some(bytes) => {
                    let mut written = 0;
                    while written < bytes.len() {
                        match writer.write(&bytes[written..]) {
                            Ok(0) => {
                                return WriteLoopOutcome::Error(PtyError::Io(
                                    std::io::Error::new(
                                        std::io::ErrorKind::WriteZero,
                                        "pty writer accepted zero bytes",
                                    ),
                                ));
                            }
                            Ok(n) => written += n,
                            Err(e) => return WriteLoopOutcome::Error(e),
                        }
                    }
                    if let Err(e) = writer.flush() {
                        return WriteLoopOutcome::Error(e);
                    }
                }
                None => return WriteLoopOutcome::InputClosed,
            }
        }
    });

    WriteLoop { input: tx, task }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::domain::pty_adapter::{PtyAdapter, SpawnRequest};
    use crate::domain::terminal_types::{PtySize, ShellSpec};
    use crate::infrastructure::mock_pty_adapter::MockPtyAdapter;

    fn fixture_request() -> SpawnRequest {
        SpawnRequest::new(
            ShellSpec::new(PathBuf::from("/bin/zsh")),
            PtySize::new(24, 80),
        )
    }

    #[tokio::test]
    async fn read_loop_streams_chunks_then_signals_eof() {
        let adapter = MockPtyAdapter::new();
        adapter.push_read_chunk(b"hello".to_vec());
        adapter.push_read_chunk(b" world".to_vec());

        let mut handle = adapter.spawn(fixture_request()).unwrap();
        let reader = handle.take_reader().unwrap();
        let mut loop_handle = spawn_read_loop(reader);

        let chunk1 = loop_handle.bytes.recv().await.unwrap();
        let chunk2 = loop_handle.bytes.recv().await.unwrap();
        assert_eq!(chunk1, b"hello");
        assert_eq!(chunk2, b" world");

        let outcome = loop_handle.task.await.unwrap();
        assert!(matches!(outcome, ReadLoopOutcome::Eof));
    }

    #[tokio::test]
    async fn write_loop_forwards_bytes_to_pty_writer() {
        let adapter = MockPtyAdapter::new();
        let mut handle = adapter.spawn(fixture_request()).unwrap();
        let writer = handle.take_writer().unwrap();
        let loop_handle = spawn_write_loop(writer);

        loop_handle.input.send(b"ls\n".to_vec()).await.unwrap();
        loop_handle.input.send(b"pwd\n".to_vec()).await.unwrap();
        drop(loop_handle.input);

        let outcome = loop_handle.task.await.unwrap();
        assert!(matches!(outcome, WriteLoopOutcome::InputClosed));
        let state = adapter.state();
        assert_eq!(state.written, b"ls\npwd\n");
        assert_eq!(state.flushed, 2);
    }
}
