use std::time::SystemTime;
use uuid::Uuid;

/// Classification of audit events for filtering and routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditEventType {
    Session,
    Feature,
    Command,
    Storage,
    Permission,
    System,
}

/// Outcome of an audited action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditStatus {
    Success,
    Failure,
    Denied,
    Pending,
}

/// A single immutable audit record capturing who did what, when, and with what result.
///
/// The `details` field is expected to be pre-redacted by the caller — it must never
/// contain secrets, tokens, or PII.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditRecord {
    pub timestamp: SystemTime,
    pub event_type: AuditEventType,
    pub session_id: Option<Uuid>,
    pub actor: String,
    pub action: String,
    pub status: AuditStatus,
    /// Pre-redacted details string. Callers must sanitize before construction.
    pub details: String,
}

impl AuditRecord {
    pub fn new(
        event_type: AuditEventType,
        actor: impl Into<String>,
        action: impl Into<String>,
        status: AuditStatus,
    ) -> Self {
        Self {
            timestamp: SystemTime::now(),
            event_type,
            session_id: None,
            actor: actor.into(),
            action: action.into(),
            status,
            details: String::new(),
        }
    }

    pub fn with_session_id(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = details.into();
        self
    }
}

/// Sink for emitted audit records. Implementations must be thread-safe.
pub trait AuditSink: Send + Sync {
    fn emit(&self, record: AuditRecord);
}

/// Helper that optionally forwards a record to the `tracing` crate when the
/// `tracing` feature is enabled. Sinks can call this inside their `emit`
/// implementation to get structured log integration.
#[cfg(feature = "tracing")]
pub fn trace_record(record: &AuditRecord) {
    tracing::info!(
        target: "openspace::audit",
        timestamp = ?record.timestamp,
        event_type = ?record.event_type,
        session_id = ?record.session_id,
        actor = %record.actor,
        action = %record.action,
        status = ?record.status,
        details = %record.details,
        "audit"
    );
}

#[cfg(not(feature = "tracing"))]
pub fn trace_record(_record: &AuditRecord) {}
