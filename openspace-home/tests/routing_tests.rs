use openspace_core::app_command::{AppCommand, FeatureCommand, StorageCommand};
use openspace_core::app_event::{AppEvent, FeatureLifecycleState};
use openspace_core::core_errors::CoreError;
use openspace_core::permission::PermissionProfile;
use openspace_core::session::{SessionDescriptor, SessionMode};
use openspace_home::application::app_router::AppRouter;
use openspace_home::infrastructure::mock_feature_runtime::MockFeatureRuntime;

#[test]
fn create_session_emits_event_and_sets_active() {
    let mut router = AppRouter::new();
    let cmd = AppCommand::CreateSession {
        project_folder: std::path::PathBuf::from("/tmp/test"),
        descriptor: SessionDescriptor::new("test"),
    };
    let events = router.apply(cmd);
    assert!(events
        .iter()
        .any(|e| matches!(e, AppEvent::SessionCreated { .. })));
    assert!(router.active_session().is_some());
}

#[test]
fn switch_mode_emits_mode_changed() {
    let mut router = AppRouter::new();
    let create = AppCommand::CreateSession {
        project_folder: std::path::PathBuf::from("/tmp/test"),
        descriptor: SessionDescriptor::new("test"),
    };
    router.apply(create);
    let session_id = router.active_session().unwrap().id;

    let cmd = AppCommand::SwitchMode {
        session_id,
        mode: SessionMode::Chat,
    };
    let events = router.apply(cmd);
    assert!(events
        .iter()
        .any(|e| matches!(e, AppEvent::ModeChanged { .. })));
    assert_eq!(router.active_session().unwrap().mode, SessionMode::Chat);
}

#[test]
fn close_session_removes_and_emits_closed() {
    let mut router = AppRouter::new();
    let create = AppCommand::CreateSession {
        project_folder: std::path::PathBuf::from("/tmp/test"),
        descriptor: SessionDescriptor::new("test"),
    };
    router.apply(create);
    let session_id = router.active_session().unwrap().id;

    let cmd = AppCommand::CloseSession { session_id };
    let events = router.apply(cmd);
    assert!(events
        .iter()
        .any(|e| matches!(e, AppEvent::SessionClosed { .. })));
    assert!(router.active_session().is_none());
}

#[test]
fn mock_runtime_survives_mode_switch() {
    let mut router = AppRouter::new();
    let create = AppCommand::CreateSession {
        project_folder: std::path::PathBuf::from("/tmp/test"),
        descriptor: SessionDescriptor::new("test"),
    };
    router.apply(create);
    let session_id = router.active_session().unwrap().id;

    let mock = MockFeatureRuntime::new("test-feature");
    router.runtime_manager.register(Box::new(mock.clone()));

    router.apply(AppCommand::SwitchMode {
        session_id,
        mode: SessionMode::Chat,
    });
    router.apply(AppCommand::SwitchMode {
        session_id,
        mode: SessionMode::Editor,
    });

    let state = mock.state();
    assert_eq!(state.activate_count, 2);
    assert_eq!(state.deactivate_count, 2);
    assert_eq!(state.last_session_id, Some(session_id));
}

#[test]
fn update_permission_emits_permission_changed() {
    let mut router = AppRouter::new();
    let create = AppCommand::CreateSession {
        project_folder: std::path::PathBuf::from("/tmp/test"),
        descriptor: SessionDescriptor::new("test"),
    };
    router.apply(create);
    let session_id = router.active_session().unwrap().id;

    let cmd = AppCommand::UpdatePermission {
        session_id,
        profile: PermissionProfile::FullAccess,
    };
    let events = router.apply(cmd);
    assert!(events
        .iter()
        .any(|e| matches!(e, AppEvent::PermissionChanged { .. })));
    assert_eq!(
        router.active_session().unwrap().permission,
        PermissionProfile::FullAccess
    );
}

#[test]
fn dispatch_to_feature_emits_lifecycle() {
    let mut router = AppRouter::new();
    let create = AppCommand::CreateSession {
        project_folder: std::path::PathBuf::from("/tmp/test"),
        descriptor: SessionDescriptor::new("test"),
    };
    router.apply(create);

    let mock = MockFeatureRuntime::new("editor-feature");
    router.runtime_manager.register(Box::new(mock.clone()));

    let cmd = AppCommand::DispatchToFeature {
        feature_id: "editor-feature".to_string(),
        command: FeatureCommand::Activate,
    };
    let events = router.apply(cmd);
    assert!(events.iter().any(|e| matches!(
        e,
        AppEvent::FeatureLifecycle {
            feature_id,
            state: FeatureLifecycleState::Ready,
        } if feature_id == "editor-feature"
    )));
    assert_eq!(mock.state().command_count, 1);
}

#[test]
fn dispatch_to_missing_feature_emits_error() {
    let mut router = AppRouter::new();
    let create = AppCommand::CreateSession {
        project_folder: std::path::PathBuf::from("/tmp/test"),
        descriptor: SessionDescriptor::new("test"),
    };
    router.apply(create);

    let cmd = AppCommand::DispatchToFeature {
        feature_id: "missing".to_string(),
        command: FeatureCommand::Activate,
    };
    let events = router.apply(cmd);
    assert!(events.iter().any(|e| matches!(
        e,
        AppEvent::Error {
            error: CoreError::FeatureNotFound(_),
            ..
        }
    )));
}

#[test]
fn storage_commands_emit_audit_events() {
    let mut router = AppRouter::new();
    let create = AppCommand::CreateSession {
        project_folder: std::path::PathBuf::from("/tmp/test"),
        descriptor: SessionDescriptor::new("test"),
    };
    router.apply(create);
    let session_id = router.active_session().unwrap().id;

    let save = AppCommand::Storage(StorageCommand::SaveSession { session_id });
    let events = router.apply(save);
    assert!(events.iter().any(|e| matches!(
        e,
        AppEvent::Audit { action, .. } if action == "session_saved"
    )));
}
