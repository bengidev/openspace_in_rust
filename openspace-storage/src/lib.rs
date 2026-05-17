pub mod migrations;
pub mod storage_error;
pub mod storage_handle;

pub use storage_error::StorageError;
pub use storage_handle::StorageHandle;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use openspace_core::permission::PermissionProfile;
    use openspace_core::session::{Session, SessionDescriptor, SessionMode};

    use super::*;

    #[tokio::test]
    async fn test_session_roundtrip_default() {
        let storage = StorageHandle::new_in_memory().unwrap();
        let descriptor = SessionDescriptor::new("test-session");
        let session = Session::new(PathBuf::from("/tmp/project"), descriptor);

        storage.write_session(&session).await.unwrap();
        let read_back = storage.read_session(session.id).await.unwrap().unwrap();

        assert_eq!(session.id, read_back.id);
        assert_eq!(session.mode, read_back.mode);
        assert_eq!(session.permission, read_back.permission);
        assert_eq!(session.project_folder, read_back.project_folder);
        assert_eq!(session.descriptor, read_back.descriptor);
    }

    #[tokio::test]
    async fn test_session_roundtrip_all_variants() {
        let storage = StorageHandle::new_in_memory().unwrap();

        for mode in [SessionMode::Terminal, SessionMode::Chat, SessionMode::Editor] {
            for permission in [
                PermissionProfile::Default,
                PermissionProfile::AutoReview,
                PermissionProfile::FullAccess,
                PermissionProfile::Custom {
                    rules: vec!["read".to_string(), "write".to_string()],
                },
            ] {
                let mut session =
                    Session::new(PathBuf::from("/home/user/project"), SessionDescriptor::new("variant-test"));
                session.mode = mode;
                session.permission = permission;

                storage.write_session(&session).await.unwrap();
                let read_back = storage.read_session(session.id).await.unwrap().unwrap();

                assert_eq!(session.id, read_back.id);
                assert_eq!(session.mode, read_back.mode);
                assert_eq!(session.permission, read_back.permission);
                assert_eq!(session.project_folder, read_back.project_folder);
                assert_eq!(session.descriptor, read_back.descriptor);
            }
        }
    }

    #[tokio::test]
    async fn test_read_missing_session() {
        let storage = StorageHandle::new_in_memory().unwrap();
        let id = uuid::Uuid::new_v4();
        let result = storage.read_session(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_migrations_run_on_init() {
        let storage = StorageHandle::new_in_memory().unwrap();
        let conn = storage.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
