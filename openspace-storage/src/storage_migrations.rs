use rusqlite::Connection;

const MIGRATIONS: &[(i64, &str, &str)] = &[(1, "v1_sessions", MIGRATION_001)];

const MIGRATION_001: &str = r#"
    CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY NOT NULL,
        mode TEXT NOT NULL,
        permission_profile TEXT NOT NULL,
        project_path TEXT NOT NULL,
        descriptor_json TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
"#;

pub fn run_migrations(conn: &mut Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    let latest: i64 = conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM _migrations", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    for &(id, name, sql) in MIGRATIONS {
        if id > latest {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO _migrations (id, name) VALUES (?1, ?2)",
                rusqlite::params![id, name],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}
