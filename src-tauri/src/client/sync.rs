use crate::config::{is_self_serving, load_app_settings};
use crate::db::{get_remote_replica_db_path, init_db_schema, prune_logs};
use crate::server::types::{HistorySyncResponse, ServerBroadcastEvent, SnapshotResponse};
use futures_util::StreamExt;
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::connect_async;

const SYNC_RETRY_INTERVAL: Duration = Duration::from_secs(5);
const SERVER_STATUS_EVENT_NAME: &str = "server-status-tick";
const PRINTER_STATUS_EVENT_NAME: &str = "printer-status-tick";

pub fn start_client_sync_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        loop {
            let settings = match load_app_settings(&app) {
                Ok(s) => s,
                Err(err) => {
                    eprintln!("Failed to load app settings in client sync worker: {err}");
                    tokio::time::sleep(SYNC_RETRY_INTERVAL).await;
                    continue;
                }
            };

            if is_self_serving(&settings.target_host) {
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }

            let replica_path = match get_remote_replica_db_path(
                &app,
                &settings.target_host,
                settings.target_port,
            ) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("Failed to resolve remote replica DB path: {err}");
                    tokio::time::sleep(SYNC_RETRY_INTERVAL).await;
                    continue;
                }
            };

            if let Err(err) = init_replica_db(&replica_path) {
                eprintln!("Failed to initialize replica DB at {:?}: {err}", replica_path);
                tokio::time::sleep(SYNC_RETRY_INTERVAL).await;
                continue;
            }

            let base_url = format!("http://{}:{}", settings.target_host, settings.target_port);
            let ws_url = format!("ws://{}:{}/api/v1/stream", settings.target_host, settings.target_port);

            match fetch_initial_snapshot(&http_client, &base_url).await {
                Ok(snapshot) => {
                    if let Err(err) = write_snapshot_to_replica(&replica_path, snapshot) {
                        eprintln!("Failed to write snapshot to replica DB: {err}");
                    } else {
                        let _ = app.emit("snapshot-synced", ());
                    }
                }
                Err(err) => {
                    eprintln!("Initial snapshot sync failed with {base_url}: {err}");
                    tokio::time::sleep(SYNC_RETRY_INTERVAL).await;
                    continue;
                }
            }

            let (max_srv, max_prn) = get_max_log_ids(&replica_path);
            match fetch_history_logs(&http_client, &base_url, max_srv, max_prn).await {
                Ok(history) => {
                    if let Err(err) = write_history_to_replica(&replica_path, history) {
                        eprintln!("Failed to write history logs to replica DB: {err}");
                    }
                }
                Err(err) => {
                    eprintln!("Historical log sync failed with {base_url}: {err}");
                }
            }

            if let Ok(conn) = Connection::open(&replica_path) {
                let _ = prune_logs(&conn, settings.retention_days, settings.max_log_entries);
            }

            if let Err(err) = stream_websocket_ticks(&ws_url, &http_client, &base_url, &replica_path, &app).await {
                eprintln!("WebSocket stream disconnected from {ws_url}: {err}");
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

fn init_replica_db(replica_path: &Path) -> Result<(), String> {
    let conn = Connection::open(replica_path).map_err(|e| e.to_string())?;
    init_db_schema(&conn)
}

fn get_max_log_ids(replica_path: &Path) -> (i64, i64) {
    let Ok(conn) = Connection::open(replica_path) else {
        return (0, 0);
    };

    let max_srv = conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM server_logs", [], |r| r.get(0))
        .unwrap_or(0);
    let max_prn = conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM printer_logs", [], |r| r.get(0))
        .unwrap_or(0);

    (max_srv, max_prn)
}

async fn fetch_initial_snapshot(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<SnapshotResponse, String> {
    let url = format!("{base_url}/api/v1/sync/snapshot");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Snapshot request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Server returned HTTP {}", resp.status()));
    }

    resp.json()
        .await
        .map_err(|e| format!("Failed to parse snapshot response: {e}"))
}

fn write_snapshot_to_replica(
    replica_path: &Path,
    snapshot: SnapshotResponse,
) -> Result<(), String> {
    let mut conn = Connection::open(replica_path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA busy_timeout = 5000; PRAGMA journal_mode = WAL;")
        .map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let srv_ids: std::collections::HashSet<String> = snapshot.servers.iter().map(|s| s.id.clone()).collect();
    for (idx, s) in snapshot.servers.into_iter().enumerate() {
        let agent_port = s.agent.as_ref().map(|a| a.port as i64);
        let agent_api = s.agent.as_ref().map(|a| a.api_key.as_str());
        tx.execute(
            "INSERT INTO servers (id, host, label, description, agent_port, agent_api_key, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                host = excluded.host,
                label = excluded.label,
                description = excluded.description,
                agent_port = excluded.agent_port,
                agent_api_key = excluded.agent_api_key,
                sort_order = excluded.sort_order",
            params![s.id, s.host, s.label, s.description, agent_port, agent_api, idx as i64],
        )
        .map_err(|e| e.to_string())?;
    }
    let existing_servers = crate::server_monitor::db::get_servers(&tx).map_err(|e| e.to_string())?;
    for existing in existing_servers {
        if !srv_ids.contains(&existing.id) {
            tx.execute("DELETE FROM servers WHERE id = ?1", params![existing.id])
                .map_err(|e| e.to_string())?;
        }
    }

    let prn_ids: std::collections::HashSet<String> = snapshot.printers.iter().map(|p| p.id.clone()).collect();
    for (idx, p) in snapshot.printers.into_iter().enumerate() {
        tx.execute(
            "INSERT INTO printers (id, host, label, description, model, location, snmp_community, web_url, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                host = excluded.host,
                label = excluded.label,
                description = excluded.description,
                model = excluded.model,
                location = excluded.location,
                snmp_community = excluded.snmp_community,
                web_url = excluded.web_url,
                sort_order = excluded.sort_order",
            params![p.id, p.host, p.label, p.description, p.model, p.location, p.snmp_community, p.web_url, idx as i64],
        )
        .map_err(|e| e.to_string())?;
    }
    let existing_printers = crate::printer_dashboard::db::get_printers(&tx).map_err(|e| e.to_string())?;
    for existing in existing_printers {
        if !prn_ids.contains(&existing.id) {
            tx.execute("DELETE FROM printers WHERE id = ?1", params![existing.id])
                .map_err(|e| e.to_string())?;
        }
    }

    tx.execute("DELETE FROM printer_filter_tags", []).map_err(|e| e.to_string())?;
    for (idx, tag) in snapshot.printer_filter_tags.all_tags.into_iter().enumerate() {
        tx.execute(
            "INSERT INTO printer_filter_tags (tag, mode, sort_order) VALUES (?1, 'all', ?2)",
            params![tag, idx as i64],
        )
        .map_err(|e| e.to_string())?;
    }
    for (idx, tag) in snapshot.printer_filter_tags.any_tags.into_iter().enumerate() {
        tx.execute(
            "INSERT INTO printer_filter_tags (tag, mode, sort_order) VALUES (?1, 'any', ?2)",
            params![tag, idx as i64],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

async fn fetch_history_logs(
    client: &reqwest::Client,
    base_url: &str,
    after_server_id: i64,
    after_printer_id: i64,
) -> Result<HistorySyncResponse, String> {
    let url = format!(
        "{base_url}/api/v1/sync/history?after_server_log_id={after_server_id}&after_printer_log_id={after_printer_id}&limit=1000"
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("History request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Server returned HTTP {}", resp.status()));
    }

    resp.json()
        .await
        .map_err(|e| format!("Failed to parse history response: {e}"))
}

fn write_history_to_replica(
    replica_path: &Path,
    history: HistorySyncResponse,
) -> Result<(), String> {
    let mut conn = Connection::open(replica_path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    for log in history.server_logs {
        tx.execute(
            "INSERT OR IGNORE INTO server_logs (id, server_id, event_type, latency_ms, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![log.id, log.server_id, log.event_type, log.latency_ms.map(|l| l as i64), log.timestamp],
        )
        .map_err(|e| e.to_string())?;
    }

    for log in history.printer_logs {
        tx.execute(
            "INSERT OR IGNORE INTO printer_logs (id, printer_id, event_type, latency_ms, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![log.id, log.printer_id, log.event_type, log.latency_ms.map(|l| l as i64), log.timestamp],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

async fn stream_websocket_ticks(
    ws_url: &str,
    http_client: &reqwest::Client,
    base_url: &str,
    replica_path: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .map_err(|e| format!("WebSocket connection failed: {e}"))?;

    let (_, mut read) = ws_stream.split();

    while let Some(msg_result) = read.next().await {
        let msg = msg_result.map_err(|e| format!("WebSocket read error: {e}"))?;
        if !msg.is_text() {
            continue;
        }

        let Ok(text) = msg.to_text() else {
            continue;
        };

        if let Ok(event) = serde_json::from_str::<ServerBroadcastEvent>(text) {
            match event {
                ServerBroadcastEvent::ServerTick(payload) => {
                    let _ = app.emit(SERVER_STATUS_EVENT_NAME, &payload);
                    let (max_srv, max_prn) = get_max_log_ids(replica_path);
                    if let Ok(history) = fetch_history_logs(http_client, base_url, max_srv, max_prn).await {
                        if !history.server_logs.is_empty() || !history.printer_logs.is_empty() {
                            let _ = write_history_to_replica(replica_path, history);
                        }
                    }
                }
                ServerBroadcastEvent::PrinterTick(payload) => {
                    let _ = app.emit(PRINTER_STATUS_EVENT_NAME, &payload);
                    let (max_srv, max_prn) = get_max_log_ids(replica_path);
                    if let Ok(history) = fetch_history_logs(http_client, base_url, max_srv, max_prn).await {
                        if !history.server_logs.is_empty() || !history.printer_logs.is_empty() {
                            let _ = write_history_to_replica(replica_path, history);
                        }
                    }
                }
                ServerBroadcastEvent::EntitiesUpdated => {
                    if let Ok(snapshot) = fetch_initial_snapshot(http_client, base_url).await {
                        if let Ok(()) = write_snapshot_to_replica(replica_path, snapshot) {
                            let (max_srv, max_prn) = get_max_log_ids(replica_path);
                            if let Ok(history) = fetch_history_logs(http_client, base_url, max_srv, max_prn).await {
                                let _ = write_history_to_replica(replica_path, history);
                            }
                            let _ = app.emit("snapshot-synced", ());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
