pub mod platform_paths;
pub mod platform_shell;

pub use platform_shell::{ResolvedShell, ShellSource, resolve_default_shell, resolve_default_shell_with};
