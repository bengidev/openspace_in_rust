//! `openspace-lsp` — language-server feature crate.
//!
//! Layered along Clean Architecture lines:
//!
//! * [`domain`] — protocol message + diagnostic value types.
//! * [`application`] — server lifecycle and diagnostic
//!   stream aggregation.
//! * [`infrastructure`] — child-process + JSON-RPC adapters.
//! * [`presenter`] — diagnostic surfaces.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presenter;
