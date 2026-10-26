use super::auth::{authenticate_and_create_session, validate_admin_token};
use super::types::{HistorySyncResponse, LoginRequest, LoginResponse, ServerBroadcastEvent, SnapshotResponse};
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
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub struct ServerAppState {
    pub db: Mutex<Connection>,
    pub broadcast_tx: broadcast::Sender<ServerBroadcastEvent>,
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

fn check_admin_auth(headers: &HeaderMap, conn: &Connection) -> Result<(), (StatusCode, String)> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .map(str::trim)
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization scheme".to_string()))?;

    match validate_admin_token(conn, token) {
        Ok(true) => Ok(()),
        Ok(false) => Err((StatusCode::FORBIDDEN, "Admin privileges required".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

async fn get_snapshot(
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<SnapshotResponse>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let servers = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let printers = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let printer_filter_tags =
        get_printer_filter_tags(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

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

async fn login(
    State(state): State<Arc<ServerAppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match authenticate_and_create_session(&conn, &payload.username, &payload.password) {
        Ok(Some((token, role))) => Ok(Json(LoginResponse {
            token,
            username: payload.username,
            role,
        })),
        Ok(None) => Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

async fn handle_post_server(
    headers: HeaderMap,
    State(state): State<Arc<ServerAppState>>,
    Json(server): Json<Server>,
) -> Result<Json<Vec<Server>>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    check_admin_auth(&headers, &conn)?;
    insert_server(&conn, &server).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let servers = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(servers))
}

async fn handle_put_server(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<ServerAppState>>,
    Json(server): Json<Server>,
) -> Result<Json<Vec<Server>>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    check_admin_auth(&headers, &conn)?;
    update_server_entry(&conn, &id, &server).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let servers = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(servers))
}

async fn handle_delete_server(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<Vec<Server>>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    check_admin_auth(&headers, &conn)?;
    delete_server_entry(&conn, &id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let servers = get_servers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(servers))
}

async fn handle_post_printer(
    headers: HeaderMap,
    State(state): State<Arc<ServerAppState>>,
    Json(printer): Json<Printer>,
) -> Result<Json<Vec<Printer>>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    check_admin_auth(&headers, &conn)?;
    insert_printer(&conn, &printer).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let printers = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(printers))
}

async fn handle_put_printer(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<ServerAppState>>,
    Json(printer): Json<Printer>,
) -> Result<Json<Vec<Printer>>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    check_admin_auth(&headers, &conn)?;
    update_printer_entry(&conn, &id, &printer).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let printers = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(printers))
}

async fn handle_delete_printer(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<ServerAppState>>,
) -> Result<Json<Vec<Printer>>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    check_admin_auth(&headers, &conn)?;
    delete_printer_entry(&conn, &id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let printers = get_printers(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(printers))
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
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/servers", post(handle_post_server))
        .route("/api/v1/servers/{id}", put(handle_put_server).delete(handle_delete_server))
        .route("/api/v1/printers", post(handle_post_printer))
        .route("/api/v1/printers/{id}", put(handle_put_printer).delete(handle_delete_printer))
        .route("/api/v1/stream", get(ws_handler))
        .with_state(state)
}
