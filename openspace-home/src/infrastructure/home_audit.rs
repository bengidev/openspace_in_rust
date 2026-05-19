//! Audit sink adapters for the home stage.
//!
//! [`NoopAuditSink`] silently drops all records — the production
//! default until the persistent audit log is wired up.
//! [`MemoryAuditSink`] captures records in insertion order so
//! tests can assert against them.

use std::sync::{Arc, Mutex};

use openspace_core::audit::{AuditRecord, AuditSink};

/// Default production sink — silently drops all audit records.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopAuditSink;

impl AuditSink for NoopAuditSink {
    fn emit(&self, _record: AuditRecord) {
        // Intentionally empty: production builds drop audit
        // records.
    }
}

/// Test-only sink that captures records in insertion order.
#[derive(Debug, Clone, Default)]
pub struct MemoryAuditSink {
    records: Arc<Mutex<Vec<AuditRecord>>>,
}

impl MemoryAuditSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot of captured records in order.
    pub fn records(&self) -> Vec<AuditRecord> {
        self.records.lock().unwrap().clone()
    }

    /// Clears all captured records.
    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
    }

    /// Returns the number of captured records.
    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AuditSink for MemoryAuditSink {
    fn emit(&self, record: AuditRecord) {
        openspace_core::audit::trace_record(&record);
        self.records.lock().unwrap().push(record);
    }
}
