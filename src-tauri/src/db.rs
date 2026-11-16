use crate:: net::event_name_for_status;
use crate::server_monitor::types::HealthStatus;
use rusqlite::{Connection, OptionalExtension, params};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub const DEFAULT_LOG_LIMIT: i64 = 50;

const APP_DIR_NAME: &str = "caffeine";
const DB_FILE_NAME: &str = "caffeine.db";

pub struct DbState(pub Mutex<Connection>);

#[cfg(target_os = "windows")]
pub fn ensure_machine_dir_permissions(dir: &Path) -> Result<(), String> {
    let status = crate::process::new_hidden_command("icacls")
        .arg(dir)
        .arg("/grant")
        .arg("*S-1-5-32-545:(OI)(CI)M")
        .arg("/t")
        .arg("/c")
        .arg("/q")
        .status()
        .map_err(|e| format!("Failed to invoke icacls on {dir:?}: {e}"))?;

    if !status.success() {
        return Err(format!("icacls failed on {dir:?} with status {status}"));
    }

    Ok(())
}

#[cfg(unix)]
pub fn ensure_machine_dir_permissions(dir: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions on machine directory {dir:?}: {e}"))
}

#[cfg(not(any(target_os = "windows", unix)))]
pub fn ensure_machine_dir_permissions(_dir: &Path) -> Result<(), String> {
    Ok(())
}

pub fn get_machine_db_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let program_data =
            std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".to_string());
        Ok(PathBuf::from(program_data).join(APP_DIR_NAME))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(PathBuf::from("/Library/Application Support").join(APP_DIR_NAME))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok(PathBuf::from("/var/lib").join(APP_DIR_NAME))
    }
}

pub fn ensure_machine_db_dir() -> Result<PathBuf, String> {
    let dir = get_machine_db_dir()?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create machine database directory at {dir:?}: {e}"))?;
    ensure_machine_dir_permissions(&dir)?;
    Ok(dir)
}

pub fn get_machine_db_path() -> Result<PathBuf, String> {
    Ok(get_machine_db_dir()?.join(DB_FILE_NAME))
}

pub fn get_remote_replica_db_path(
    app: &AppHandle,
    host: &str,
    port: u16,
) -> Result<PathBuf, String> {
    let local_data_dir = app
        .path()
        .local_data_dir()
        .map_err(|e| format!("Failed to resolve local data dir: {e}"))?;

    let slug = crate::config::sanitize_host_slug(host, port);
    let dir = local_data_dir.join(APP_DIR_NAME).join("remotes").join(slug);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create remote replica directory at {dir:?}: {e}"))?;

    Ok(dir.join(DB_FILE_NAME))
}

pub fn get_local_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let local_data_dir = app
        .path()
        .local_data_dir()
        .map_err(|e| format!("Failed to resolve local data dir: {e}"))?;

    let base_dir = local_data_dir.join(APP_DIR_NAME);
    fs::create_dir_all(&base_dir)
        .map_err(|e| format!("Failed to create database directory: {e}"))?;

    Ok(base_dir.join(DB_FILE_NAME))
}

pub fn resolve_active_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let settings = crate::config::load_app_settings(app)?;
    if settings.server_mode {
        Ok(ensure_machine_db_dir()?.join(DB_FILE_NAME))
    } else if !crate::config::is_self_serving(&settings.target_host) {
        get_remote_replica_db_path(app, &settings.target_host, settings.target_port)
    } else {
        get_local_db_path(app)
    }
}

pub fn switch_active_db(app: &AppHandle, db_state: &DbState) -> Result<(), String> {
    let new_path = resolve_active_db_path(app)?;
    let new_conn = Connection::open(&new_path)
        .map_err(|e| format!("Failed to open database at {new_path:?}: {e}"))?;
    init_db_schema(&new_conn)?;

    let mut conn_guard = db_state
        .0
        .lock()
        .map_err(|e| format!("Database lock poisoned: {e}"))?;
    *conn_guard = new_conn;
    Ok(())
}

fn get_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    resolve_active_db_path(app)
}

pub fn init_db_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA busy_timeout = 5000;
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS servers (
            id TEXT PRIMARY KEY,
            host TEXT NOT NULL,
            label TEXT NOT NULL,
            description TEXT,
            agent_port INTEGER,
            agent_api_key TEXT,
            ssh_port INTEGER,
            rdp_port INTEGER,
            remote_username TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS server_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            server_id TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
            event_type TEXT NOT NULL,
            latency_ms INTEGER,
            timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_server_logs_lookup
        ON server_logs(server_id, timestamp DESC);

        CREATE TABLE IF NOT EXISTS printers (
            id TEXT PRIMARY KEY,
            host TEXT NOT NULL,
            label TEXT NOT NULL,
            description TEXT,
            model TEXT,
            location TEXT,
            snmp_community TEXT NOT NULL DEFAULT 'public',
            web_url TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS printer_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            printer_id TEXT NOT NULL REFERENCES printers(id) ON DELETE CASCADE,
            event_type TEXT NOT NULL,
            latency_ms INTEGER,
            timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_printer_logs_lookup
        ON printer_logs(printer_id, timestamp DESC);

        CREATE TABLE IF NOT EXISTS printer_filter_tags (
            tag TEXT NOT NULL,
            mode TEXT NOT NULL DEFAULT 'all',
            sort_order INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (tag, mode)
        );
        ",
    )
    .map_err(|e| format!("Failed to run schema initialization: {e}"))?;

    Ok(())
}

pub fn init_db(app: &AppHandle) -> Result<DbState, String> {
    let db_path = get_db_path(app)?;
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {e}"))?;

    init_db_schema(&conn)?;

    Ok(DbState(Mutex::new(conn)))
}

pub fn ensure_entity_id(id: &str) -> String {
    if id.trim().is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        id.to_string()
    }
}

pub fn next_sort_order(conn: &Connection, table: &str) -> Result<i64, String> {
    let query = format!("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM {table}");
    conn.query_row(&query, [], |r| r.get(0))
        .map_err(|e| format!("Failed to calculate next sort order for {table}: {e}"))
}

pub fn delete_by_id(conn: &Connection, table: &str, id: &str) -> Result<(), String> {
    let sql = format!("DELETE FROM {table} WHERE id = ?1");
    conn.execute(&sql, params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_table_sort_order<T, F>(
    conn: &mut Connection,
    table: &str,
    items: &[T],
    get_id: F,
) -> Result<(), String>
where
    F: Fn(&T) -> &str,
{
    let sql = format!("UPDATE {table} SET sort_order = ?1 WHERE id = ?2");
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx.prepare(&sql).map_err(|e| e.to_string())?;
        for (idx, item) in items.iter().enumerate() {
            stmt.execute(params![idx as i64, get_id(item)])
                .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn insert_log_entry(
    conn: &Connection,
    table: &str,
    fk_column: &str,
    entity_id: &str,
    event_type: &str,
    latency_ms: Option<u64>,
) -> Result<String, String> {
    let insert_sql =
        format!("INSERT INTO {table} ({fk_column}, event_type, latency_ms) VALUES (?1, ?2, ?3)");
    conn.execute(
        &insert_sql,
        params![entity_id, event_type, latency_ms.map(|l| l as i64)],
    )
    .map_err(|e| e.to_string())?;

    let select_sql = format!("SELECT timestamp FROM {table} WHERE id = last_insert_rowid()");
    conn.query_row(&select_sql, [], |r| r.get(0))
        .map_err(|e| e.to_string())
}

pub fn get_latest_log_entry(
    conn: &Connection,
    table: &str,
    fk_column: &str,
    entity_id: &str,
) -> Result<Option<(String, String)>, String> {
    let sql = format!(
        "SELECT event_type, timestamp FROM {table}
         WHERE {fk_column} = ?1
         ORDER BY id DESC LIMIT 1"
    );
    conn.query_row(&sql, params![entity_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })
    .optional()
    .map_err(|e| e.to_string())
}

pub fn get_latest_connectivity_log_entry(
    conn: &Connection,
    table: &str,
    fk_column: &str,
    entity_id: &str,
) -> Result<Option<(String, String)>, String> {
    let sql = format!(
        "SELECT event_type, timestamp FROM {table}
         WHERE {fk_column} = ?1 AND event_type IN ('connected', 'disconnected')
         ORDER BY id DESC LIMIT 1"
    );
    conn.query_row(&sql, params![entity_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })
    .optional()
    .map_err(|e| e.to_string())
}

pub fn query_log_entries<T, F>(
    conn: &Connection,
    table: &str,
    fk_column: &str,
    entity_id: &str,
    limit: i64,
    map_fn: F,
) -> Result<Vec<T>, String>
where
    F: Fn(i64, String, String, Option<u64>, String) -> T,
{
    let sql = format!(
        "SELECT id, {fk_column}, event_type, latency_ms, timestamp
         FROM {table}
         WHERE {fk_column} = ?1
         ORDER BY id DESC
         LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![entity_id, limit], |row| {
            let lat: Option<i64> = row.get(3)?;
            Ok(map_fn(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                lat.map(|l| l as u64),
                row.get(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn resolve_initial_event(
    db: &DbState,
    table: &str,
    fk_column: &str,
    entity_id: &str,
    current_event: &str,
    latency_ms: Option<u64>,
) -> Result<(String, String), String> {
    let conn = db
        .0
        .lock()
        .map_err(|e| format!("Database lock poisoned: {e}"))?;

    if let Some((ev, at)) = get_latest_connectivity_log_entry(&conn, table, fk_column, entity_id)? {
        if ev == current_event {
            return Ok((ev, at));
        }
    }

    let at = insert_log_entry(&conn, table, fk_column, entity_id, current_event, latency_ms)?;
    Ok((current_event.to_string(), at))
}

pub fn record_status_transition(
    db: &DbState,
    table: &str,
    fk_column: &str,
    entity_id: &str,
    previous_status: &HealthStatus,
    new_status: &HealthStatus,
    latency_ms: Option<u64>,
) -> Result<Option<(String, String)>, String> {
    let was_connected = *previous_status != HealthStatus::Offline;
    let is_connected = *new_status != HealthStatus::Offline;
    if was_connected == is_connected {
        return Ok(None);
    }

    let new_event = event_name_for_status(new_status);
    let conn = db
        .0
        .lock()
        .map_err(|e| format!("Database lock poisoned: {e}"))?;
    let ts = insert_log_entry(&conn, table, fk_column, entity_id, new_event, latency_ms)?;
    Ok(Some((new_event.to_string(), ts)))
}

pub fn prune_logs(
    conn: &Connection,
    retention_days: u32,
    max_entries_per_entity: u32,
) -> Result<(), String> {
    let days_param = format!("-{retention_days} days");

    conn.execute(
        "DELETE FROM server_logs WHERE timestamp < datetime('now', ?1)",
        params![days_param],
    )
    .map_err(|e| format!("Failed to prune server_logs by age: {e}"))?;

    conn.execute(
        "DELETE FROM printer_logs WHERE timestamp < datetime('now', ?1)",
        params![days_param],
    )
    .map_err(|e| format!("Failed to prune printer_logs by age: {e}"))?;

    conn.execute(
        "DELETE FROM server_logs
         WHERE id NOT IN (
             SELECT id FROM (
                 SELECT id, ROW_NUMBER() OVER (PARTITION BY server_id ORDER BY id DESC) as rn
                 FROM server_logs
             ) WHERE rn <= ?1
         )",
        params![max_entries_per_entity],
    )
    .map_err(|e| format!("Failed to prune server_logs by entry count: {e}"))?;

    conn.execute(
        "DELETE FROM printer_logs
         WHERE id NOT IN (
             SELECT id FROM (
                 SELECT id, ROW_NUMBER() OVER (PARTITION BY printer_id ORDER BY id DESC) as rn
                 FROM printer_logs
             ) WHERE rn <= ?1
         )",
        params![max_entries_per_entity],
    )
    .map_err(|e| format!("Failed to prune printer_logs by entry count: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_logs_retains_within_threshold() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE server_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                latency_ms INTEGER,
                timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            );
            CREATE TABLE printer_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                printer_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                latency_ms INTEGER,
                timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            );
            ",
        )
        .unwrap();

        for i in 1..=10 {
            conn.execute(
                "INSERT INTO server_logs (server_id, event_type, timestamp) VALUES ('srv1', 'connected', datetime('now', ?1))",
                params![format!("-{} days", i)],
            )
            .unwrap();
        }

        prune_logs(&conn, 5, 100).unwrap();

        let count: i64 = conn
            .query_row("SELECT count(*) FROM server_logs", [], |r| r.get(0))
            .unwrap();
        assert!(count <= 5);

        for _ in 1..=10 {
            conn.execute(
                "INSERT INTO server_logs (server_id, event_type) VALUES ('srv1', 'connected')",
                [],
            )
            .unwrap();
        }

        prune_logs(&conn, 30, 4).unwrap();
        let count_after_max: i64 = conn
            .query_row("SELECT count(*) FROM server_logs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count_after_max, 4);
    }

    #[test]
    fn test_resolve_initial_event_filters_non_connectivity_events() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE printer_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                printer_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                latency_ms INTEGER,
                timestamp TEXT NOT NULL DEFAULT '2026-10-23T08:00:00Z'
            );
            ",
        )
        .unwrap();

        insert_log_entry(&conn, "printer_logs", "printer_id", "p1", "connected", Some(5)).unwrap();
        insert_log_entry(&conn, "printer_logs", "printer_id", "p1", "toner empty", None).unwrap();

        let db = DbState(Mutex::new(conn));
        let (ev, at) = resolve_initial_event(&db, "printer_logs", "printer_id", "p1", "connected", Some(3)).unwrap();
        assert_eq!(ev, "connected");
        assert_eq!(at, "2026-10-23T08:00:00Z");

        let (ev2, _) = resolve_initial_event(&db, "printer_logs", "printer_id", "p1", "disconnected", None).unwrap();
        assert_eq!(ev2, "disconnected");
    }

    #[test]
    fn test_get_machine_db_dir_and_path_resolve_expected_suffix() {
        let dir = get_machine_db_dir().unwrap();
        assert_eq!(dir.file_name().and_then(|n| n.to_str()), Some(APP_DIR_NAME));

        let db_path = get_machine_db_path().unwrap();
        assert_eq!(db_path, dir.join(DB_FILE_NAME));
    }

    #[test]
    fn test_ensure_machine_dir_permissions_succeeds_on_temp_dir() {
        let unique = format!(
            "caffeine_db_perm_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        fs::create_dir_all(&dir).unwrap();

        let res = ensure_machine_dir_permissions(&dir);
        assert!(res.is_ok(), "ensure_machine_dir_permissions failed: {res:?}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o755);
        }

        let _ = fs::remove_dir_all(&dir);
    }
}
