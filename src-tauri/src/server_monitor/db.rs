use super::types::{Agent, Server, ServerLog};
use crate::db::{
    delete_by_id, ensure_entity_id, insert_log_entry, next_sort_order, query_log_entries,
    update_table_sort_order,
};
use rusqlite::{Connection, params};

pub const SERVER_LOGS_TABLE: &str = "server_logs";
pub const SERVER_ID_COLUMN: &str = "server_id";
const SERVERS_TABLE: &str = "servers";

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
            let agent = port.zip(key).map(|(port, api_key)| Agent { port, api_key });

            Ok(Server {
                id: row.get(0)?,
                host: row.get(1)?,
                label: row.get(2)?,
                description: row.get(3)?,
                agent,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn insert_server(conn: &Connection, server: &Server) -> Result<(), String> {
    let id = ensure_entity_id(&server.id);
    let next_order = next_sort_order(conn, SERVERS_TABLE)?;
    let port = server.agent.as_ref().map(|a| a.port as i64);
    let key = server.agent.as_ref().map(|a| a.api_key.as_str());

    conn.execute(
        "INSERT INTO servers (id, host, label, description, agent_port, agent_api_key, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            server.host,
            server.label,
            server.description,
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
    delete_by_id(conn, SERVERS_TABLE, id)
}

pub fn update_sort_order(conn: &mut Connection, servers: &[Server]) -> Result<(), String> {
    update_table_sort_order(conn, SERVERS_TABLE, servers, |s| s.id.as_str())
}

pub fn add_log_entry(
    conn: &Connection,
    server_id: &str,
    event_type: &str,
    latency_ms: Option<u64>,
) -> Result<String, String> {
    insert_log_entry(
        conn,
        SERVER_LOGS_TABLE,
        SERVER_ID_COLUMN,
        server_id,
        event_type,
        latency_ms,
    )
}

pub fn query_logs(
    conn: &Connection,
    server_id: &str,
    limit: i64,
) -> Result<Vec<ServerLog>, String> {
    query_log_entries(
        conn,
        SERVER_LOGS_TABLE,
        SERVER_ID_COLUMN,
        server_id,
        limit,
        |id, server_id, event_type, latency_ms, timestamp| ServerLog {
            id,
            server_id,
            event_type,
            latency_ms,
            timestamp,
        },
    )
}
