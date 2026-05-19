//! Integration tests for the PTY backend slice (issue #38).
//!
//! These tests exercise the real `portable-pty`-backed adapter and
//! the async I/O loops together. They run under a tokio runtime
//! and spawn short-lived /bin/sh commands, so they need a working
//! PTY (every macOS dev machine; most Linux CI containers).
//!
//! Acceptance criteria targeted:
//!
//! 1. Real PTY spawn produces stdout bytes through the read loop.
//! 2. Keyboard input flows through the write loop into the child.
//! 3. Dropping the reader/writer halves does NOT kill the child;
//!    the runtime-owned handle keeps the process alive across a
//!    simulated CenterSurface remount (mode switch survival).

use std::path::PathBuf;
use std::time::Duration;

use openspace_terminal::application::{
    ReadLoopOutcome, WriteLoopOutcome, spawn_read_loop, spawn_write_loop,
};
use openspace_terminal::domain::pty_adapter::{PtyAdapter, SpawnRequest};
use openspace_terminal::domain::terminal_types::{PtySize, ShellSpec};
use openspace_terminal::infrastructure::PortablePtyAdapter;

fn sh_spec() -> ShellSpec {
    // Minimal env: PATH so /bin/sh can find /bin/echo / /bin/sleep
    // (some shells lookup `echo` even when it's a builtin),
    // TERM so the shell does not warn about a missing terminfo
    // entry. No secrets inherited because env_clear() runs in the
    // adapter.
    ShellSpec::new(PathBuf::from("/bin/sh"))
        .with_env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .with_env("TERM", "xterm-256color")
}

fn small_size() -> PtySize {
    PtySize::new(24, 80)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_pty_spawn_streams_stdout_bytes() {
    let adapter = PortablePtyAdapter::new();
    let spec = sh_spec().with_arg("-c").with_arg("echo openspace-pty");
    let request = SpawnRequest::new(spec, small_size());

    let mut handle = adapter.spawn(request).expect("spawn /bin/sh");
    let reader = handle.take_reader().expect("reader available");
    let _writer = handle.take_writer().expect("writer available");

    let mut read_loop = spawn_read_loop(reader);

    // Drain bytes until the read loop signals EOF. We collect into
    // a string and assert the stdout marker is present; the shell
    // may emit extra control bytes around the echoed line.
    let mut collected: Vec<u8> = Vec::new();
    let drain = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(chunk) = read_loop.bytes.recv().await {
            collected.extend_from_slice(&chunk);
            if collected.windows(b"openspace-pty".len()).any(|w| w == b"openspace-pty") {
                break;
            }
        }
    });
    drain.await.expect("read drained within timeout");

    let text = String::from_utf8_lossy(&collected);
    assert!(
        text.contains("openspace-pty"),
        "expected echoed marker in PTY stdout, got: {text:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_pty_write_loop_delivers_keystrokes() {
    let adapter = PortablePtyAdapter::new();
    // `cat` echoes everything written to its stdin to stdout,
    // which lets us prove the write loop reaches the child.
    let spec = sh_spec().with_arg("-c").with_arg("cat");
    let request = SpawnRequest::new(spec, small_size());

    let mut handle = adapter.spawn(request).expect("spawn /bin/sh -c cat");
    let reader = handle.take_reader().expect("reader available");
    let writer = handle.take_writer().expect("writer available");

    let mut read_loop = spawn_read_loop(reader);
    let write_loop = spawn_write_loop(writer);

    write_loop
        .input
        .send(b"openspace-input\n".to_vec())
        .await
        .expect("send to write loop");

    let mut collected: Vec<u8> = Vec::new();
    let drain = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(chunk) = read_loop.bytes.recv().await {
            collected.extend_from_slice(&chunk);
            if collected
                .windows(b"openspace-input".len())
                .any(|w| w == b"openspace-input")
            {
                break;
            }
        }
    });
    drain.await.expect("read drained within timeout");

    assert!(
        String::from_utf8_lossy(&collected).contains("openspace-input"),
        "expected cat to echo the keystrokes, got: {:?}",
        String::from_utf8_lossy(&collected),
    );

    // Tear the cat process down — kill is the only termination
    // path. Drop is non-destructive.
    handle.kill().expect("kill cat");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dropping_reader_and_writer_does_not_kill_child() {
    // Acceptance criterion #5 (mode switch survival): the
    // CenterSurface holds the reader / writer halves through the
    // I/O loops. The home runtime owns the PtyHandle. Switching
    // modes drops the view (and therefore the loops); the handle
    // stays put and the child must keep running.

    let adapter = PortablePtyAdapter::new();
    // sleep 3 outlives the test's drop-and-assert window.
    let spec = sh_spec().with_arg("-c").with_arg("sleep 3");
    let request = SpawnRequest::new(spec, small_size());

    let mut handle = adapter.spawn(request).expect("spawn /bin/sh -c sleep");
    assert!(handle.is_alive(), "child alive immediately after spawn");

    {
        let reader = handle.take_reader().expect("reader available");
        let writer = handle.take_writer().expect("writer available");
        let read_loop = spawn_read_loop(reader);
        let write_loop = spawn_write_loop(writer);

        // Drop the loop handles. This drops the channel ends; the
        // worker tasks will exit on their next iteration (read
        // loop on receiver-hangup / write loop on sender-drop).
        drop(read_loop);
        drop(write_loop);
    }

    // Give the worker tasks a moment to notice the drops. If the
    // adapter were destructive, the child would be reaped here.
    tokio::time::sleep(Duration::from_millis(150)).await;

    assert!(
        handle.is_alive(),
        "PTY child must survive view-side drop (mode switch survival)",
    );

    // Clean up so the test does not leave a sleep process behind.
    handle.kill().expect("kill sleep");
}

#[tokio::test]
async fn read_loop_outcome_is_eof_when_child_exits() {
    // Spawning `true` runs and exits immediately. The read loop
    // should see EOF and report ReadLoopOutcome::Eof.
    let adapter = PortablePtyAdapter::new();
    let spec = sh_spec().with_arg("-c").with_arg("true");
    let request = SpawnRequest::new(spec, small_size());

    let mut handle = adapter.spawn(request).expect("spawn /bin/sh -c true");
    let reader = handle.take_reader().expect("reader available");
    let _writer = handle.take_writer();

    let read_loop = spawn_read_loop(reader);

    // Drain channel; ignore content, we only care about the loop
    // outcome.
    drop(read_loop.bytes);
    let outcome = tokio::time::timeout(Duration::from_secs(5), read_loop.task)
        .await
        .expect("read loop joined")
        .expect("task did not panic");

    assert!(
        matches!(outcome, ReadLoopOutcome::Eof),
        "expected Eof outcome, got {outcome:?}"
    );
}

#[tokio::test]
async fn write_loop_outcome_is_input_closed_on_sender_drop() {
    let adapter = PortablePtyAdapter::new();
    let spec = sh_spec().with_arg("-c").with_arg("cat");
    let request = SpawnRequest::new(spec, small_size());

    let mut handle = adapter.spawn(request).expect("spawn /bin/sh -c cat");
    let _reader = handle.take_reader();
    let writer = handle.take_writer().expect("writer available");

    let write_loop = spawn_write_loop(writer);
    drop(write_loop.input);

    let outcome = tokio::time::timeout(Duration::from_secs(5), write_loop.task)
        .await
        .expect("write loop joined")
        .expect("task did not panic");

    assert!(
        matches!(outcome, WriteLoopOutcome::InputClosed),
        "expected InputClosed outcome, got {outcome:?}"
    );

    handle.kill().expect("kill cat");
}
