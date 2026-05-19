pub mod core_app_command;
pub mod core_app_event;
pub mod core_audit;
pub mod core_command_palette;
pub mod core_errors;
pub mod core_permission;
pub mod core_session;
pub mod core_types;

// Stable public API names retained at crate root for ergonomic
// imports — call sites can pick either path. New code should
// prefer the prefixed module path so the file layout stays
// discoverable.
pub use core_app_command as app_command;
pub use core_app_event as app_event;
pub use core_audit as audit;
pub use core_command_palette as command_palette;
pub use core_permission as permission;
pub use core_session as session;
