use super::types::{HistorySyncResponse, ServerBroadcastEvent, SnapshotResponse};
use crate::printer_dashboard::db::{
    delete_printer_entry, get_printer_filter_tags, get_printers, insert_printer,
    update_printer_entry,
};
use crate::printer_dashboard::types::Printer;
use crate::server_monitor::db::{
    delete_server_entry, get_servers, insert_server, update_server_entry,
};
use crate::server_monitor::types::Server;
use axum::{
    body::Bytes,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

pub struct ServerAppState {
    pub db: Mutex<Connection>,
    pub broadcast_tx: broadcast::Sender<ServerBroadcastEvent>,
    pub app: Option<AppHandle>,
    pub nonce_cache: Mutex<super::auth::NonceCache>,
    pub server_tokens: Option<Vec<crate::config::ServerTokenEntry>>,
}

fn notify_entities_updated(state: &ServerAppState) {
    let _ = state.broadcast_tx.send(ServerBroadcastEvent::EntitiesUpdated);
    if let Some(app) = &state.app {
        let _ = app.emit("snapshot-synced", ());
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub after_server_log_id: Option<i64>,
    pub after_printer_log_id: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub limit: Option<i64>,
}

fn check_admin_auth(
    headers: &HeaderMap,
    method: &str,
    uri_path: &str,
    body: &[u8],
    state: &ServerAppState,
) -> Result<(), (StatusCode, String)> {
    if headers.contains_key("Authorization") {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Plaintext Bearer tokens are prohibited. HMAC-SHA256 request signing is required.".to_string(),
        ));
    }

    let ts_str = headers
        .get("X-Caffeine-Timestamp")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing X-Caffeine-Timestamp header".to_string()))?;

    let timestamp = ts_str
        .parse::<u64>()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid X-Caffeine-Timestamp header format".to_string()))?;

    let nonce = headers
        .get("X-Caffeine-Nonce")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing X-Caffeine-Nonce header".to_string()))?;

    if nonce.trim().is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Empty X-Caffeine-Nonce header".to_string()));
    }

    let signature = headers
        .get("X-Caffeine-Signature")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing X-Caffeine-Signature header".to_string()))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .as_secs();

    let mut nonce_guard = state
        .nonce_cache
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Nonce cache lock error: {e}")))?;

    nonce_guard
        .check_and_record(nonce, timestamp, now)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let active_tokens: Vec<String> = if let Some(override_tokens) = &state.server_tokens {
        override_tokens.iter().map(|t| t.token.clone()).collect()
    } else {
        crate::config::load_machine_settings()
            .ok()
            .map(|m| m.server_tokens.into_iter().map(|t| t.token).collect())
            .unwrap_or_default()
    };

    if active_tokens.is_empty() {
        return Err((
            StatusCode::FORBIDDEN,
            "Server has no admin tokens configured".to_string(),
        ));
    }

    let mut authorized = false;
    for token in &active_tokens {
        if token.trim().is_empty() {
            continue;
        }
        match super::auth::validate_hmac_signature(
            token.trim(),
            method,
            uri_path,
            timestamp,
            nonce,
            body,
            signature,
        ) {
            Ok(true) => {
                authorized = true;
                break;
            }
            Ok(false) => {}
            Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
        }
    }

    if !authorized {
        return Err((StatusCode::FORBIDDEN, "Invalid HMAC-SHA256 signature".to_string()));
    }

    Ok(())
}

async fn get_snapshot(
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<SnapshotResponse>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut servers = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let mut printers = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let printer_filter_tags =
        get_printer_filter_tags(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    for server in &mut servers {
        if let Some(ref mut agent) = server.agent {
            agent.api_key = String::new();
        }
    }

    for printer in &mut printers {
        printer.snmp_community = super::types::REDACTED_SECRET.to_string();
    }

    Ok(Json(SnapshotResponse {
        servers,
        printers,
        printer_filter_tags,
    }))
}

async fn get_history(
    Query(query): Query<HistoryQuery>,
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<HistorySyncResponse>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let limit = query.limit.unwrap_or(500);
    let after_server = query.after_server_log_id.unwrap_or(0);
    let after_printer = query.after_printer_log_id.unwrap_or(0);

    let mut server_stmt = conn
        .prepare(
            "SELECT id, server_id, event_type, latency_ms, timestamp
             FROM server_logs WHERE id > ?1 ORDER BY id ASC LIMIT ?2",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let server_logs = server_stmt
        .query_map(rusqlite::params![after_server, limit], |row| {
            let lat: Option<i64> = row.get(3)?;
            Ok(crate::server_monitor::types::ServerLog {
                id: row.get(0)?,
                server_id: row.get(1)?,
                event_type: row.get(2)?,
                latency_ms: lat.map(|l| l as u64),
                timestamp: row.get(4)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut printer_stmt = conn
        .prepare(
            "SELECT id, printer_id, event_type, latency_ms, timestamp
             FROM printer_logs WHERE id > ?1 ORDER BY id ASC LIMIT ?2",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let printer_logs = printer_stmt
        .query_map(rusqlite::params![after_printer, limit], |row| {
            let lat: Option<i64> = row.get(3)?;
            Ok(crate::printer_dashboard::types::PrinterLog {
                id: row.get(0)?,
                printer_id: row.get(1)?,
                event_type: row.get(2)?,
                latency_ms: lat.map(|l| l as u64),
                timestamp: row.get(4)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(HistorySyncResponse {
        server_logs,
        printer_logs,
    }))
}

async fn handle_post_server(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    State(state): State<Arc<ServerAppState>>,
    bytes: Bytes,
) -> Result<Json<Vec<Server>>, (StatusCode, String)> {
    check_admin_auth(&headers, method.as_str(), uri.path(), &bytes, &state)?;
    let server: Server = serde_json::from_slice(&bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid server payload: {e}")))?;
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    insert_server(&conn, &server).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    notify_entities_updated(&state);
    let servers = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(servers))
}

async fn handle_put_server(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<ServerAppState>>,
    bytes: Bytes,
) -> Result<Json<Vec<Server>>, (StatusCode, String)> {
    check_admin_auth(&headers, method.as_str(), uri.path(), &bytes, &state)?;
    let server: Server = serde_json::from_slice(&bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid server payload: {e}")))?;
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    update_server_entry(&conn, &id, &server).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    notify_entities_updated(&state);
    let servers = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(servers))
}

async fn handle_delete_server(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<Vec<Server>>, (StatusCode, String)> {
    check_admin_auth(&headers, method.as_str(), uri.path(), &[], &state)?;
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    delete_server_entry(&conn, &id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    notify_entities_updated(&state);
    let servers = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(servers))
}

async fn handle_reorder_servers(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    State(state): State<Arc<ServerAppState>>,
    bytes: Bytes,
) -> Result<Json<Vec<Server>>, (StatusCode, String)> {
    check_admin_auth(&headers, method.as_str(), uri.path(), &bytes, &state)?;
    let servers: Vec<Server> = serde_json::from_slice(&bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid servers payload: {e}")))?;
    let mut conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    crate::server_monitor::db::update_sort_order(&mut conn, &servers)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    notify_entities_updated(&state);
    let updated = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(updated))
}

async fn handle_get_server_logs(
    Path(id): Path<String>,
    Query(query): Query<LogQuery>,
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<Vec<crate::server_monitor::types::ServerLog>>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let limit = query.limit.unwrap_or(crate::db::DEFAULT_LOG_LIMIT);
    let logs = crate::server_monitor::db::query_logs(&conn, &id, limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(logs))
}

async fn handle_post_printer(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    State(state): State<Arc<ServerAppState>>,
    bytes: Bytes,
) -> Result<Json<Vec<Printer>>, (StatusCode, String)> {
    check_admin_auth(&headers, method.as_str(), uri.path(), &bytes, &state)?;
    let printer: Printer = serde_json::from_slice(&bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid printer payload: {e}")))?;
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    insert_printer(&conn, &printer).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    notify_entities_updated(&state);
    let printers = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(printers))
}

async fn handle_put_printer(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<ServerAppState>>,
    bytes: Bytes,
) -> Result<Json<Vec<Printer>>, (StatusCode, String)> {
    check_admin_auth(&headers, method.as_str(), uri.path(), &bytes, &state)?;
    let printer: Printer = serde_json::from_slice(&bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid printer payload: {e}")))?;
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    update_printer_entry(&conn, &id, &printer).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    notify_entities_updated(&state);
    let printers = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(printers))
}

async fn handle_delete_printer(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<Vec<Printer>>, (StatusCode, String)> {
    check_admin_auth(&headers, method.as_str(), uri.path(), &[], &state)?;
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    delete_printer_entry(&conn, &id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    notify_entities_updated(&state);
    let printers = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(printers))
}

async fn handle_reorder_printers(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    State(state): State<Arc<ServerAppState>>,
    bytes: Bytes,
) -> Result<Json<Vec<Printer>>, (StatusCode, String)> {
    check_admin_auth(&headers, method.as_str(), uri.path(), &bytes, &state)?;
    let printers: Vec<Printer> = serde_json::from_slice(&bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid printers payload: {e}")))?;
    let mut conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    crate::printer_dashboard::db::update_printer_sort_order(&mut conn, &printers)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    notify_entities_updated(&state);
    let updated = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(updated))
}

async fn handle_get_printer_logs(
    Path(id): Path<String>,
    Query(query): Query<LogQuery>,
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<Vec<crate::printer_dashboard::types::PrinterLog>>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let limit = query.limit.unwrap_or(crate::db::DEFAULT_LOG_LIMIT);
    let logs = crate::printer_dashboard::db::query_printer_logs(&conn, &id, limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(logs))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerAppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_stream(socket, state))
}

async fn handle_ws_stream(mut socket: axum::extract::ws::WebSocket, state: Arc<ServerAppState>) {
    let mut rx = state.broadcast_tx.subscribe();
    while let Ok(event) = rx.recv().await {
        let Ok(json) = serde_json::to_string(&event) else {
            continue;
        };
        if socket.send(axum::extract::ws::Message::Text(json.into())).await.is_err() {
            break;
        }
    }
}

pub fn create_router(state: Arc<ServerAppState>) -> Router {
    Router::new()
        .route("/api/v1/sync/snapshot", get(get_snapshot))
        .route("/api/v1/sync/history", get(get_history))
        .route("/api/v1/servers", post(handle_post_server))
        .route("/api/v1/servers/reorder", post(handle_reorder_servers))
        .route("/api/v1/servers/{id}", put(handle_put_server).delete(handle_delete_server))
        .route("/api/v1/servers/{id}/logs", get(handle_get_server_logs))
        .route("/api/v1/printers", post(handle_post_printer))
        .route("/api/v1/printers/reorder", post(handle_reorder_printers))
        .route("/api/v1/printers/{id}", put(handle_put_printer).delete(handle_delete_printer))
        .route("/api/v1/printers/{id}/logs", get(handle_get_printer_logs))
        .route("/api/v1/stream", get(ws_handler))
        .with_state(state)
}
