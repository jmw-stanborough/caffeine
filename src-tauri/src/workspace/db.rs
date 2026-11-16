use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::AppHandle;

const WORKSPACE_DB_FILE: &str = "workspace.db";

pub struct WorkspaceDbState(pub Mutex<Connection>);

pub fn init_workspace_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA busy_timeout = 5000;
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS todos (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            completed_at INTEGER,
            due_at INTEGER,
            reminder_at INTEGER,
            reminder_sent INTEGER NOT NULL DEFAULT 0,
            reminder_acknowledged INTEGER NOT NULL DEFAULT 0,
            recurrence TEXT,
            created_at INTEGER NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_todos_reminder_lookup
        ON todos(completed, reminder_sent, reminder_at);
        ",
    )
    .map_err(|e| format!("Failed to initialize workspace database schema: {e}"))?;

    migrate_todos_reminder_acknowledged(conn)
}

fn migrate_todos_reminder_acknowledged(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(todos)")
        .map_err(|e| format!("Failed to inspect todos schema: {e}"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("Failed to read todos column names: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect todos column names: {e}"))?;

    if columns.iter().any(|name| name == "reminder_acknowledged") {
        return Ok(());
    }

    // Backfill historical reminders as acknowledged so upgrading does not surface past notifications as missed.
    conn.execute_batch(
        "
        ALTER TABLE todos ADD COLUMN reminder_acknowledged INTEGER NOT NULL DEFAULT 0;
        UPDATE todos SET reminder_acknowledged = 1 WHERE reminder_sent = 1 OR completed = 1;
        ",
    )
    .map_err(|e| format!("Failed to migrate todos.reminder_acknowledged column: {e}"))
}

pub fn init_workspace_db_at(roaming_dir: &Path) -> Result<WorkspaceDbState, String> {
    fs::create_dir_all(roaming_dir).map_err(|e| {
        format!(
            "Failed to create workspace roaming directory at {:?}: {e}",
            roaming_dir
        )
    })?;

    let db_path = roaming_dir.join(WORKSPACE_DB_FILE);
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open workspace database at {:?}: {e}", db_path))?;

    init_workspace_schema(&conn)?;
    Ok(WorkspaceDbState(Mutex::new(conn)))
}

pub fn init_workspace_db(app: &AppHandle) -> Result<WorkspaceDbState, String> {
    let roaming_dir = crate::config::get_user_roaming_dir(app)?;
    init_workspace_db_at(&roaming_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::temp_roaming_dir;

    #[test]
    fn test_init_workspace_db_at_creates_roaming_db_and_schema() {
        let dir = temp_roaming_dir("caffeine_workspace_db");
        let state = init_workspace_db_at(&dir).unwrap();
        assert!(dir.join(WORKSPACE_DB_FILE).is_file());

        let conn = state.0.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM todos", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        drop(conn);

        let _ = fs::remove_dir_all(&dir);
    }
}
