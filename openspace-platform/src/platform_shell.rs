//! Platform shell-resolution policy.
//!
//! V1 targets macOS. The resolver follows a small, predictable
//! ladder so the rest of the app can ask one question — "what
//! shell should I spawn?" — without scattering env-variable lookups
//! across feature crates.
//!
//! Resolution order:
//!
//! 1. `$SHELL` if it points at a non-empty path
//! 2. `getpwuid_r` lookup for the current user (macOS)
//! 3. `/bin/zsh` (the macOS default since Catalina)
//! 4. `/bin/sh` as a last resort
//!
//! The resolver only inspects the environment; it does not stat
//! the resulting path. Callers that need to spawn the shell are
//! responsible for surfacing a useful error if exec fails.

use std::ffi::OsString;
use std::path::PathBuf;

/// Source describing how the resolved shell was chosen. Useful for
/// audit logging and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellSource {
    /// Pulled from the `SHELL` environment variable.
    Environment,
    /// Looked up via `getpwuid_r` on the current user.
    PasswordDatabase,
    /// macOS default fallback (`/bin/zsh`).
    PlatformDefault,
    /// Hard-coded last-resort fallback (`/bin/sh`).
    UltimateFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedShell {
    pub path: PathBuf,
    pub source: ShellSource,
}

impl ResolvedShell {
    pub fn new(path: PathBuf, source: ShellSource) -> Self {
        Self { path, source }
    }
}

/// Resolve the shell using the real process environment.
///
/// Lives behind a thin wrapper so tests can inject a synthetic env
/// without touching `std::env`.
pub fn resolve_default_shell() -> ResolvedShell {
    let getter = |key: &str| std::env::var_os(key);
    resolve_default_shell_with(&getter, &lookup_passwd_shell)
}

/// Variant used by tests. The two closures stand in for environment
/// access and passwd-database lookup so the resolution ladder can be
/// exercised deterministically.
pub fn resolve_default_shell_with(
    env_get: &dyn Fn(&str) -> Option<OsString>,
    passwd_lookup: &dyn Fn() -> Option<PathBuf>,
) -> ResolvedShell {
    if let Some(value) = env_get("SHELL")
        && !value.is_empty()
    {
        return ResolvedShell::new(PathBuf::from(value), ShellSource::Environment);
    }

    if let Some(path) = passwd_lookup() {
        return ResolvedShell::new(path, ShellSource::PasswordDatabase);
    }

    if cfg!(target_os = "macos") {
        return ResolvedShell::new(PathBuf::from("/bin/zsh"), ShellSource::PlatformDefault);
    }

    ResolvedShell::new(PathBuf::from("/bin/sh"), ShellSource::UltimateFallback)
}

/// Real passwd-database lookup. Returns `None` on every platform in
/// v1 — `getpwuid_r` integration lives behind a future libc-backed
/// adapter. Splitting it out now keeps the resolver testable and
/// avoids a hard dependency on `libc` until we actually need it.
fn lookup_passwd_shell() -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with(pairs: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<OsString> {
        let owned: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |key: &str| {
            owned
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| OsString::from(v))
        }
    }

    #[test]
    fn shell_env_var_wins_when_set() {
        let env = env_with(&[("SHELL", "/opt/homebrew/bin/fish")]);
        let result = resolve_default_shell_with(&env, &|| None);
        assert_eq!(result.path, PathBuf::from("/opt/homebrew/bin/fish"));
        assert_eq!(result.source, ShellSource::Environment);
    }

    #[test]
    fn empty_shell_var_falls_through_to_passwd() {
        let env = env_with(&[("SHELL", "")]);
        let passwd = || Some(PathBuf::from("/usr/local/bin/zsh"));
        let result = resolve_default_shell_with(&env, &passwd);
        assert_eq!(result.path, PathBuf::from("/usr/local/bin/zsh"));
        assert_eq!(result.source, ShellSource::PasswordDatabase);
    }

    #[test]
    fn missing_shell_and_passwd_falls_through_to_platform_default() {
        let env = env_with(&[]);
        let result = resolve_default_shell_with(&env, &|| None);
        if cfg!(target_os = "macos") {
            assert_eq!(result.path, PathBuf::from("/bin/zsh"));
            assert_eq!(result.source, ShellSource::PlatformDefault);
        } else {
            assert_eq!(result.path, PathBuf::from("/bin/sh"));
            assert_eq!(result.source, ShellSource::UltimateFallback);
        }
    }

    #[test]
    fn passwd_lookup_used_when_shell_unset() {
        let env = env_with(&[]);
        let passwd = || Some(PathBuf::from("/bin/bash"));
        let result = resolve_default_shell_with(&env, &passwd);
        assert_eq!(result.path, PathBuf::from("/bin/bash"));
        assert_eq!(result.source, ShellSource::PasswordDatabase);
    }
}
