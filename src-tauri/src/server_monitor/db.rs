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
            "SELECT id, host, label, description, agent_port, agent_api_key, ssh_port, rdp_port, remote_username
             FROM servers ORDER BY sort_order ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let port: Option<u16> = row.get(4)?;
            let key: Option<String> = row.get(5)?;
            let agent = port.map(|port| Agent {
                port,
                api_key: key.unwrap_or_default(),
            });

            Ok(Server {
                id: row.get(0)?,
                host: row.get(1)?,
                label: row.get(2)?,
                description: row.get(3)?,
                agent,
                ssh_port: row.get(6)?,
                rdp_port: row.get(7)?,
                remote_username: row.get(8)?,
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
    let ssh_port = server.ssh_port.map(|p| p as i64);
    let rdp_port = server.rdp_port.map(|p| p as i64);

    conn.execute(
        "INSERT INTO servers (id, host, label, description, agent_port, agent_api_key, ssh_port, rdp_port, remote_username, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            server.host,
            server.label,
            server.description,
            port,
            key,
            ssh_port,
            rdp_port,
            server.remote_username,
            next_order
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn update_server_entry(conn: &Connection, id: &str, server: &Server) -> Result<(), String> {
    let port = server.agent.as_ref().map(|a| a.port as i64);
    let key = server
        .agent
        .as_ref()
        .map(|a| crate::server::types::sanitize_secret_update(&a.api_key));
    let ssh_port = server.ssh_port.map(|p| p as i64);
    let rdp_port = server.rdp_port.map(|p| p as i64);

    conn.execute(
        "UPDATE servers
         SET host = ?1, label = ?2, description = ?3, agent_port = ?4,
             agent_api_key = CASE WHEN ?4 IS NULL THEN NULL ELSE COALESCE(NULLIF(?5, ''), agent_api_key) END,
             ssh_port = ?6, rdp_port = ?7, remote_username = ?8
         WHERE id = ?9",
        params![
            server.host,
            server.label,
            server.description,
            port,
            key,
            ssh_port,
            rdp_port,
            server.remote_username,
            id
        ],
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE servers (
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
            ",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_get_servers_constructs_agent_with_null_or_empty_api_key() {
        let conn = setup_in_memory_db();
        conn.execute(
            "INSERT INTO servers (id, host, label, agent_port, agent_api_key, sort_order)
             VALUES ('s-null', '10.0.0.1', 'Null Key', 9001, NULL, 0),
                    ('s-empty', '10.0.0.2', 'Empty Key', 9001, '', 1)",
            [],
        )
        .unwrap();

        let servers = get_servers(&conn).unwrap();
        assert_eq!(servers.len(), 2);

        let agent_null = servers[0].agent.as_ref().expect("agent should be Some when port is set");
        assert_eq!(agent_null.port, 9001);
        assert_eq!(agent_null.api_key, "");

        let agent_empty = servers[1].agent.as_ref().expect("agent should be Some when port is set");
        assert_eq!(agent_empty.port, 9001);
        assert_eq!(agent_empty.api_key, "");
    }

    #[test]
    fn test_update_server_preserves_agent_api_key_when_redacted_or_empty() {
        let conn = setup_in_memory_db();
        let initial = Server {
            id: "s-1".to_string(),
            host: "10.0.0.10".to_string(),
            label: "Primary".to_string(),
            description: None,
            agent: Some(Agent {
                port: 9001,
                api_key: "super_secret_key".to_string(),
            }),
            ssh_port: Some(22),
            rdp_port: None,
            remote_username: Some("admin".to_string()),
        };
        insert_server(&conn, &initial).unwrap();

        let with_empty_key = Server {
            label: "Primary Edited".to_string(),
            agent: Some(Agent {
                port: 9002,
                api_key: "".to_string(),
            }),
            ..initial.clone()
        };
        update_server_entry(&conn, "s-1", &with_empty_key).unwrap();

        let loaded = get_servers(&conn).unwrap();
        assert_eq!(loaded[0].label, "Primary Edited");
        assert_eq!(loaded[0].agent.as_ref().unwrap().port, 9002);
        assert_eq!(loaded[0].agent.as_ref().unwrap().api_key, "super_secret_key");

        let with_redacted_key = Server {
            label: "Primary Redacted".to_string(),
            agent: Some(Agent {
                port: 9003,
                api_key: crate::server::types::REDACTED_SECRET.to_string(),
            }),
            ..initial.clone()
        };
        update_server_entry(&conn, "s-1", &with_redacted_key).unwrap();

        let loaded = get_servers(&conn).unwrap();
        assert_eq!(loaded[0].label, "Primary Redacted");
        assert_eq!(loaded[0].agent.as_ref().unwrap().port, 9003);
        assert_eq!(loaded[0].agent.as_ref().unwrap().api_key, "super_secret_key");
    }
}
