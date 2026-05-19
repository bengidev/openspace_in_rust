//! Builds [`SpawnRequest`]s from platform shell policy.
//!
//! The application layer asks `openspace-platform` for the default
//! shell, then layers on a deliberately small env. Provider
//! secrets, OAuth tokens, and other inherited environment data
//! never reach the PTY — the adapter calls `env_clear()` and only
//! installs what we put in [`ShellSpec::env`].
//!
//! Acceptance criterion #1 on issue #38: `$SHELL` resolution flows
//! through `openspace-platform`. Acceptance criterion #2: PTY
//! spawns with the user's shell and environment, sans secrets.

use std::path::PathBuf;

use openspace_platform::{ResolvedShell, resolve_default_shell};

use crate::domain::pty_adapter::SpawnRequest;
use crate::domain::terminal_types::{PtySize, ShellSpec};

/// Environment keys that are safe to forward to a freshly spawned
/// shell. Picked deliberately:
///
/// * `HOME`, `USER`, `LOGNAME`: shells use these to source
///   profile files and render prompts.
/// * `LANG`, `LC_ALL`, `LC_CTYPE`: keep UTF-8 locale handling.
/// * `PATH`: without it, even `/bin/ls` invocations fail in
///   restricted shells.
/// * `TERM`, `COLORTERM`: the emulator we ship is a real terminal,
///   so we want apps to enable color output.
///
/// Anything not on this list does not flow through. That includes
/// `OPENAI_API_KEY`, `AWS_*`, `GITHUB_TOKEN`, and friends.
pub const SAFE_ENV_KEYS: &[&str] = &[
    "HOME",
    "USER",
    "LOGNAME",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "PATH",
    "TERM",
    "COLORTERM",
];

/// Build a [`SpawnRequest`] using platform shell resolution and the
/// safe-env policy. Tests should use [`build_spawn_request_with`].
pub fn build_spawn_request(size: PtySize, cwd: Option<PathBuf>) -> SpawnRequest {
    let shell = resolve_default_shell();
    let getter = |key: &str| std::env::var(key).ok();
    build_spawn_request_with(shell, size, cwd, &getter)
}

/// Test-friendly variant. The caller injects a [`ResolvedShell`]
/// and an env getter so resolution and the safe-env filter are both
/// deterministic.
pub fn build_spawn_request_with(
    shell: ResolvedShell,
    size: PtySize,
    cwd: Option<PathBuf>,
    env_get: &dyn Fn(&str) -> Option<String>,
) -> SpawnRequest {
    let mut spec = ShellSpec::new(shell.path);
    // Run shells as login so PATH, prompt, and aliases match the
    // user's expectations on first launch.
    spec = spec.with_arg("-l");
    for key in SAFE_ENV_KEYS {
        if let Some(value) = env_get(key) {
            spec = spec.with_env(*key, value);
        }
    }
    let mut request = SpawnRequest::new(spec, size);
    if let Some(cwd) = cwd {
        request = request.with_cwd(cwd);
    }
    request
}

#[cfg(test)]
mod tests {
    use openspace_platform::ShellSource;

    use super::*;

    fn shell_at(path: &str) -> ResolvedShell {
        ResolvedShell::new(PathBuf::from(path), ShellSource::Environment)
    }

    fn env_with(pairs: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        let owned: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |key: &str| {
            owned
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        }
    }

    #[test]
    fn safe_env_filter_drops_provider_secrets() {
        let env = env_with(&[
            ("HOME", "/Users/test"),
            ("PATH", "/usr/bin:/bin"),
            ("OPENAI_API_KEY", "sk-secret"),
            ("AWS_SECRET_ACCESS_KEY", "very-secret"),
            ("GITHUB_TOKEN", "ghp_secret"),
            ("TERM", "xterm-256color"),
        ]);
        let request = build_spawn_request_with(
            shell_at("/bin/zsh"),
            PtySize::new(24, 80),
            None,
            &env,
        );
        assert!(request.shell.env.contains_key("HOME"));
        assert!(request.shell.env.contains_key("PATH"));
        assert!(request.shell.env.contains_key("TERM"));
        assert!(
            !request.shell.env.contains_key("OPENAI_API_KEY"),
            "provider secrets must not leak into PTY env",
        );
        assert!(!request.shell.env.contains_key("AWS_SECRET_ACCESS_KEY"));
        assert!(!request.shell.env.contains_key("GITHUB_TOKEN"));
    }

    #[test]
    fn missing_env_keys_are_silently_skipped() {
        let env = env_with(&[("HOME", "/Users/test")]);
        let request = build_spawn_request_with(
            shell_at("/bin/zsh"),
            PtySize::new(24, 80),
            None,
            &env,
        );
        assert_eq!(request.shell.env.len(), 1);
        assert_eq!(request.shell.env.get("HOME").map(String::as_str), Some("/Users/test"));
    }

    #[test]
    fn shell_runs_as_login_by_default() {
        let env = env_with(&[]);
        let request = build_spawn_request_with(
            shell_at("/bin/zsh"),
            PtySize::new(24, 80),
            None,
            &env,
        );
        assert_eq!(request.shell.args, vec!["-l".to_string()]);
        assert_eq!(request.shell.program, PathBuf::from("/bin/zsh"));
    }

    #[test]
    fn cwd_flows_into_request_when_provided() {
        let env = env_with(&[]);
        let request = build_spawn_request_with(
            shell_at("/bin/zsh"),
            PtySize::new(24, 80),
            Some(PathBuf::from("/tmp/work")),
            &env,
        );
        assert_eq!(request.cwd, Some(PathBuf::from("/tmp/work")));
    }
}
