use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use openspace_core::permission::PermissionProfile;
use openspace_core::session::{Session, SessionDescriptor, SessionMode};
use rusqlite::Connection;
use uuid::Uuid;

use crate::migrations::run_migrations;
use crate::storage_error::StorageError;

pub struct StorageHandle {
    pub(crate) conn: Arc<Mutex<Connection>>,
}

impl StorageHandle {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut conn = Connection::open(path)?;
        run_migrations(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn new_in_memory() -> Result<Self, StorageError> {
        let mut conn = Connection::open_in_memory()?;
        run_migrations(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_default() -> Result<Self, StorageError> {
        let proj_dirs =
            directories::ProjectDirs::from("com", "openspace", "openspace")
                .ok_or(StorageError::NoProjectDirs)?;
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("openspace.db");
        Self::new(&db_path)
    }

    pub async fn write_session(&self, session: &Session) -> Result<(), StorageError> {
        let conn = Arc::clone(&self.conn);
        let session = session.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = conn.lock().unwrap();
            write_session_impl(&mut conn, &session)
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::other(e)))?
    }

    pub async fn read_session(&self, id: Uuid) -> Result<Option<Session>, StorageError> {
        let conn = Arc::clone(&self.conn);
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let mut conn = conn.lock().unwrap();
            read_session_impl(&mut conn, &id_str)
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::other(e)))?
    }
}

fn mode_to_text(mode: SessionMode) -> &'static str {
    match mode {
        SessionMode::Terminal => "terminal",
        SessionMode::Chat => "chat",
        SessionMode::Editor => "editor",
    }
}

fn mode_from_text(s: &str) -> Result<SessionMode, StorageError> {
    match s {
        "terminal" => Ok(SessionMode::Terminal),
        "chat" => Ok(SessionMode::Chat),
        "editor" => Ok(SessionMode::Editor),
        _ => Err(StorageError::InvalidMode(s.to_string())),
    }
}

fn permission_to_text(profile: &PermissionProfile) -> String {
    match profile {
        PermissionProfile::Default => "default".to_string(),
        PermissionProfile::AutoReview => "auto_review".to_string(),
        PermissionProfile::FullAccess => "full_access".to_string(),
        PermissionProfile::Custom { rules } => {
            format!("custom:{}", serde_json::to_string(rules).unwrap())
        }
    }
}

fn permission_from_text(s: &str) -> Result<PermissionProfile, StorageError> {
    match s {
        "default" => Ok(PermissionProfile::Default),
        "auto_review" => Ok(PermissionProfile::AutoReview),
        "full_access" => Ok(PermissionProfile::FullAccess),
        s if s.starts_with("custom:") => {
            let json = &s[7..];
            let rules: Vec<String> = serde_json::from_str(json).map_err(StorageError::Json)?;
            Ok(PermissionProfile::Custom { rules })
        }
        _ => Err(StorageError::InvalidPermissionProfile(s.to_string())),
    }
}

fn write_session_impl(
    conn: &mut Connection,
    session: &Session,
) -> Result<(), StorageError> {
    let descriptor_json = serde_json::to_string(&session.descriptor)?;
    conn.execute(
        "INSERT OR REPLACE INTO sessions
         (id, mode, permission_profile, project_path, descriptor_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            session.id.to_string(),
            mode_to_text(session.mode),
            permission_to_text(&session.permission),
            session.project_folder.to_string_lossy(),
            descriptor_json,
        ),
    )?;
    Ok(())
}

fn read_session_impl(
    conn: &mut Connection,
    id: &str,
) -> Result<Option<Session>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT mode, permission_profile, project_path, descriptor_json
         FROM sessions WHERE id = ?1",
    )?;
    let mut rows = stmt.query([id])?;

    if let Some(row) = rows.next()? {
        let mode_text: String = row.get(0)?;
        let permission_text: String = row.get(1)?;
        let project_path_text: String = row.get(2)?;
        let descriptor_json: String = row.get(3)?;

        let mode = mode_from_text(&mode_text)?;
        let permission = permission_from_text(&permission_text)?;
        let project_folder = PathBuf::from(project_path_text);
        let descriptor: SessionDescriptor = serde_json::from_str(&descriptor_json).map_err(StorageError::Json)?;
        let id = Uuid::parse_str(id).map_err(|e| {
            StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            ))
        })?;

        Ok(Some(Session {
            id,
            mode,
            permission,
            project_folder,
            descriptor,
        }))
    } else {
        Ok(None)
    }
}
