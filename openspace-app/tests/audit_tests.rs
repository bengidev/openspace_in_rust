use openspace_app::audit::{MemoryAuditSink, NoopAuditSink};
use openspace_core::audit::{AuditEventType, AuditRecord, AuditSink, AuditStatus};
use uuid::Uuid;

#[test]
fn noop_sink_drops_records_silently() {
    let sink = NoopAuditSink;
    let record = AuditRecord::new(AuditEventType::System, "test", "noop_action", AuditStatus::Success);
    sink.emit(record); // should not panic or error
}

#[test]
fn memory_sink_captures_records_in_order() {
    let sink = MemoryAuditSink::new();

    let r1 = AuditRecord::new(AuditEventType::Session, "user", "create", AuditStatus::Success)
        .with_session_id(Uuid::new_v4())
        .with_details("project=/tmp/test1");

    let r2 = AuditRecord::new(AuditEventType::Command, "palette", "open", AuditStatus::Pending)
        .with_session_id(Uuid::new_v4())
        .with_details("query=empty");

    let r3 = AuditRecord::new(AuditEventType::Storage, "system", "save", AuditStatus::Failure)
        .with_details("disk full");

    sink.emit(r1.clone());
    sink.emit(r2.clone());
    sink.emit(r3.clone());

    let captured = sink.records();
    assert_eq!(captured.len(), 3);

    assert_eq!(captured[0].event_type, AuditEventType::Session);
    assert_eq!(captured[0].actor, "user");
    assert_eq!(captured[0].action, "create");
    assert_eq!(captured[0].status, AuditStatus::Success);
    assert!(captured[0].session_id.is_some());
    assert_eq!(captured[0].details, "project=/tmp/test1");

    assert_eq!(captured[1].event_type, AuditEventType::Command);
    assert_eq!(captured[1].actor, "palette");
    assert_eq!(captured[1].action, "open");
    assert_eq!(captured[1].status, AuditStatus::Pending);

    assert_eq!(captured[2].event_type, AuditEventType::Storage);
    assert_eq!(captured[2].actor, "system");
    assert_eq!(captured[2].action, "save");
    assert_eq!(captured[2].status, AuditStatus::Failure);
    assert!(captured[2].session_id.is_none());
    assert_eq!(captured[2].details, "disk full");
}

#[test]
fn memory_sink_preserves_ordering() {
    let sink = MemoryAuditSink::new();

    for i in 0..5 {
        let record = AuditRecord::new(
            AuditEventType::Feature,
            "router",
            format!("action_{}", i),
            AuditStatus::Success,
        );
        sink.emit(record);
    }

    let captured = sink.records();
    for (i, record) in captured.iter().enumerate() {
        assert_eq!(record.action, format!("action_{}", i));
    }
}

#[test]
fn memory_sink_clear_empties_records() {
    let sink = MemoryAuditSink::new();
    sink.emit(AuditRecord::new(AuditEventType::System, "test", "clear", AuditStatus::Success));
    assert_eq!(sink.len(), 1);

    sink.clear();
    assert!(sink.is_empty());
    assert_eq!(sink.records().len(), 0);
}

#[test]
fn audit_record_builder_methods() {
    let session_id = Uuid::new_v4();
    let record = AuditRecord::new(AuditEventType::Permission, "admin", "grant", AuditStatus::Success)
        .with_session_id(session_id)
        .with_details("profile=FullAccess");

    assert_eq!(record.event_type, AuditEventType::Permission);
    assert_eq!(record.actor, "admin");
    assert_eq!(record.action, "grant");
    assert_eq!(record.status, AuditStatus::Success);
    assert_eq!(record.session_id, Some(session_id));
    assert_eq!(record.details, "profile=FullAccess");
}

#[test]
fn audit_record_has_timestamp() {
    let before = std::time::SystemTime::now();
    let record = AuditRecord::new(AuditEventType::System, "test", "timestamp", AuditStatus::Pending);
    let after = std::time::SystemTime::now();

    assert!(record.timestamp >= before);
    assert!(record.timestamp <= after);
}
