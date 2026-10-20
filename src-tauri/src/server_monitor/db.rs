use super::types::{Agent, Server, ServerLog};
use rusqlite::{Connection, params};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct DbState(pub Mutex<Connection>);

fn get_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .config_dir()
        .map_err(|e| format!("Failed to resolve config dir: {e}"))?;

    let base_dir = config_dir.join("caffeine");
    fs::create_dir_all(&base_dir)
        .map_err(|e| format!("Failed to create database directory: {e}"))?;

    Ok(base_dir.join("caffeine.db"))
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
        ",
    )
    .map_err(|e| format!("Failed to run schema migrations: {e}"))?;

    migrate_legacy_json(app, &conn)?;

    Ok(DbState(Mutex::new(conn)))
}

fn migrate_legacy_json(app: &AppHandle, conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT count(*) FROM servers", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    let config_dir = match app.path().config_dir() {
        Ok(dir) => dir,
        Err(_) => return Ok(()),
    };

    let json_path = config_dir
        .join("caffeine")
        .join("server-monitor")
        .join("settings.json");

    if !json_path.exists() {
        return Ok(());
    }

    let contents = match fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    #[derive(serde::Deserialize)]
    struct LegacySettings {
        #[serde(default)]
        servers: Vec<Server>,
    }

    if let Ok(settings) = serde_json::from_str::<LegacySettings>(&contents) {
        for (idx, mut srv) in settings.servers.into_iter().enumerate() {
            if srv.id.is_empty() {
                srv.id = uuid::Uuid::new_v4().to_string();
            }

            let agent_port = srv.agent.as_ref().map(|a| a.port as i64);
            let agent_api = srv.agent.as_ref().map(|a| a.api_key.as_str());

            let _ = conn.execute(
                "INSERT OR IGNORE INTO servers (id, host, label, description, agent_port, agent_api_key, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    srv.id,
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

pub fn get_servers(conn: &Connection) -> Result<Vec<Server>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, host, label, description, agent_port, agent_api_key
             FROM servers ORDER BY sort_order ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let port: Option<u16> = row.get(4)?;
            let key: Option<String> = row.get(5)?;
            let agent = match (port, key) {
                (Some(p), Some(k)) => Some(Agent {
                    port: p,
                    api_key: k,
                }),
                _ => None,
            };

            Ok(Server {
                id: row.get(0)?,
                host: row.get(1)?,
                label: row.get(2)?,
                description: row.get(3)?,
                agent,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn insert_server(conn: &Connection, server: &Server) -> Result<(), String> {
    let mut srv = server.clone();
    if srv.id.is_empty() {
        srv.id = uuid::Uuid::new_v4().to_string();
    }

    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM servers",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let port = srv.agent.as_ref().map(|a| a.port as i64);
    let key = srv.agent.as_ref().map(|a| a.api_key.as_str());

    conn.execute(
        "INSERT INTO servers (id, host, label, description, agent_port, agent_api_key, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            srv.id,
            srv.host,
            srv.label,
            srv.description,
            port,
            key,
            next_order
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn update_server_entry(conn: &Connection, id: &str, server: &Server) -> Result<(), String> {
    let port = server.agent.as_ref().map(|a| a.port as i64);
    let key = server.agent.as_ref().map(|a| a.api_key.as_str());

    conn.execute(
        "UPDATE servers
         SET host = ?1, label = ?2, description = ?3, agent_port = ?4, agent_api_key = ?5
         WHERE id = ?6",
        params![server.host, server.label, server.description, port, key, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn delete_server_entry(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM servers WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_sort_order(conn: &mut Connection, servers: &[Server]) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare("UPDATE servers SET sort_order = ?1 WHERE id = ?2")
            .map_err(|e| e.to_string())?;

        for (idx, server) in servers.iter().enumerate() {
            stmt.execute(params![idx as i64, server.id])
                .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add_log_entry(
    conn: &Connection,
    server_id: &str,
    event_type: &str,
    latency_ms: Option<u64>,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO server_logs (server_id, event_type, latency_ms) VALUES (?1, ?2, ?3)",
        params![server_id, event_type, latency_ms.map(|l| l as i64)],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn query_logs(
    conn: &Connection,
    server_id: &str,
    limit: i64,
) -> Result<Vec<ServerLog>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, server_id, event_type, latency_ms, timestamp
             FROM server_logs
             WHERE server_id = ?1
             ORDER BY id DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![server_id, limit], |row| {
            let lat: Option<i64> = row.get(3)?;
            Ok(ServerLog {
                id: row.get(0)?,
                server_id: row.get(1)?,
                event_type: row.get(2)?,
                latency_ms: lat.map(|l| l as u64),
                timestamp: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for log in rows {
        logs.push(log.map_err(|e| e.to_string())?);
    }
    Ok(logs)
}
