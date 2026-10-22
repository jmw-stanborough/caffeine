use crate:: net::event_name_for_status;
use crate::server_monitor::types::{HealthStatus, Server};
use rusqlite::{Connection, OptionalExtension, params};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub const DEFAULT_LOG_LIMIT: i64 = 50;

const APP_DIR_NAME: &str = "caffeine";
const DB_FILE_NAME: &str = "caffeine.db";
const LEGACY_MONITOR_DIR: &str = "server-monitor";
const LEGACY_SETTINGS_FILE: &str = "settings.json";

pub struct DbState(pub Mutex<Connection>);

fn get_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .config_dir()
        .map_err(|e| format!("Failed to resolve config dir: {e}"))?;

    let base_dir = config_dir.join(APP_DIR_NAME);
    fs::create_dir_all(&base_dir)
        .map_err(|e| format!("Failed to create database directory: {e}"))?;

    Ok(base_dir.join(DB_FILE_NAME))
}

pub fn init_db(app: &AppHandle) -> Result<DbState, String> {
    let db_path = get_db_path(app)?;
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {e}"))?;

    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS servers (
            id TEXT PRIMARY KEY,
            host TEXT NOT NULL,
            label TEXT NOT NULL,
            description TEXT,
            agent_port INTEGER,
            agent_api_key TEXT,
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
    .map_err(|e| format!("Failed to run schema migrations: {e}"))?;

    migrate_printer_filter_tags_table(&conn)?;
    migrate_legacy_json(app, &conn)?;

    Ok(DbState(Mutex::new(conn)))
}

/// Earlier versions stored `printer_filter_tags` with `tag` as the sole primary key.
/// Rebuild the table with a composite `(tag, mode)` key if the `mode` column is missing.
fn migrate_printer_filter_tags_table(conn: &Connection) -> Result<(), String> {
    let has_mode: bool = {
        let mut stmt = conn
            .prepare("PRAGMA table_info(printer_filter_tags)")
            .map_err(|e| e.to_string())?;
        let cols = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?;
        cols.filter_map(Result::ok).any(|name| name == "mode")
    };

    if !has_mode {
        conn.execute_batch(
            "
            CREATE TABLE printer_filter_tags_new (
                tag TEXT NOT NULL,
                mode TEXT NOT NULL DEFAULT 'all',
                sort_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (tag, mode)
            );
            INSERT OR IGNORE INTO printer_filter_tags_new (tag, mode, sort_order)
            SELECT tag, 'all', sort_order FROM printer_filter_tags;
            DROP TABLE printer_filter_tags;
            ALTER TABLE printer_filter_tags_new RENAME TO printer_filter_tags;
            ",
        )
        .map_err(|e| format!("Failed to migrate printer_filter_tags schema: {e}"))?;
    }

    Ok(())
}

/// Imports servers from the pre-SQLite `settings.json` file on first run so existing
/// user configurations are preserved across upgrades.
fn migrate_legacy_json(app: &AppHandle, conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT count(*) FROM servers", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    let Ok(config_dir) = app.path().config_dir() else {
        return Ok(());
    };

    let json_path = config_dir
        .join(APP_DIR_NAME)
        .join(LEGACY_MONITOR_DIR)
        .join(LEGACY_SETTINGS_FILE);

    let Ok(contents) = fs::read_to_string(&json_path) else {
        return Ok(());
    };

    #[derive(serde::Deserialize)]
    struct LegacySettings {
        #[serde(default)]
        servers: Vec<Server>,
    }

    if let Ok(settings) = serde_json::from_str::<LegacySettings>(&contents) {
        for (idx, srv) in settings.servers.into_iter().enumerate() {
            let id = ensure_entity_id(&srv.id);
            let agent_port = srv.agent.as_ref().map(|a| a.port as i64);
            let agent_api = srv.agent.as_ref().map(|a| a.api_key.as_str());

            let _ = conn.execute(
                "INSERT OR IGNORE INTO servers (id, host, label, description, agent_port, agent_api_key, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    srv.host,
                    srv.label,
                    srv.description,
                    agent_port,
                    agent_api,
                    idx as i64
                ],
            );
        }
    }

    Ok(())
}

pub fn ensure_entity_id(id: &str) -> String {
    if id.trim().is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        id.to_string()
    }
}

pub fn next_sort_order(conn: &Connection, table: &str) -> i64 {
    let query = format!("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM {table}");
    conn.query_row(&query, [], |r| r.get(0)).unwrap_or(0)
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

/// Retrieves the most recent persisted connection event for an entity, or records an initial
/// event if none exists yet.
pub fn resolve_initial_event(
    db: &DbState,
    table: &str,
    fk_column: &str,
    entity_id: &str,
    default_event: &str,
    latency_ms: Option<u64>,
) -> (String, String) {
    let Ok(conn) = db.0.lock() else {
        return (default_event.to_string(), String::new());
    };

    if let Some((ev, at)) = get_latest_log_entry(&conn, table, fk_column, entity_id).unwrap_or(None)
    {
        (ev, at)
    } else {
        let at = insert_log_entry(&conn, table, fk_column, entity_id, default_event, latency_ms)
            .unwrap_or_default();
        (default_event.to_string(), at)
    }
}

/// Logs a connection state transition when an entity changes between reachable and offline.
pub fn record_status_transition(
    db: &DbState,
    table: &str,
    fk_column: &str,
    entity_id: &str,
    previous_status: &HealthStatus,
    new_status: &HealthStatus,
    latency_ms: Option<u64>,
) -> Option<(String, String)> {
    let was_connected = *previous_status != HealthStatus::Offline;
    let is_connected = *new_status != HealthStatus::Offline;
    if was_connected == is_connected {
        return None;
    }

    let new_event = event_name_for_status(new_status);
    let conn = db.0.lock().ok()?;
    let ts = insert_log_entry(&conn, table, fk_column, entity_id, new_event, latency_ms).ok()?;
    Some((new_event.to_string(), ts))
}
