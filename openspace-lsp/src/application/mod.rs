//! LSP feature application layer.
//!
//! Language-server lifecycle orchestration: launch, JSON-RPC
//! framing, diagnostic stream aggregation. The raw protocol
//! transport lives in the infrastructure layer once it lands.

pub mod lsp_client;
