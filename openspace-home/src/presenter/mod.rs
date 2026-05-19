//! Home feature presenter layer.
//!
//! Iced view + reducer for the workspace shell, the center
//! surface placeholders, and the command palette overlay.
//!
//! State mutation routes through `shell_update`; rendering goes
//! through `shell_view`. Both are public so the host application
//! can splice them into its own router without depending on the
//! internal reducer signature.

pub mod app_shell;
pub mod center_surface;
pub mod command_palette_overlay;

pub use app_shell::{
    AppShell, Message as HomeMessage, run, shell_subscription, shell_update, shell_view,
};
pub use command_palette_overlay::{CommandPaletteOverlay, PaletteMessage};
