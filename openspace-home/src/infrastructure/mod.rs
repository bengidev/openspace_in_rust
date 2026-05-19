//! Home feature infrastructure layer.
//!
//! Concrete adapters used by the home stage: audit sinks (no-op
//! production sink + in-memory test sink) and the mock feature
//! runtime used by routing/integration tests.

pub mod home_audit;
pub mod mock_feature_runtime;

pub use home_audit::{MemoryAuditSink, NoopAuditSink};
pub use mock_feature_runtime::{MockFeatureRuntime, MockFeatureState};
