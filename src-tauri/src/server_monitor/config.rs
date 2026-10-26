use super::db::{
    delete_server_entry, get_servers, insert_server, query_logs, update_server_entry,
    update_sort_order,
};
use super::types::{Server, ServerLog};
use crate::db::{DEFAULT_LOG_LIMIT, DbState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn load_servers(db: State<'_, DbState>) -> Result<Vec<Server>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    get_servers(&conn)
}

#[tauri::command]
pub async fn save_server(
    app: AppHandle,
    db: State<'_, DbState>,
    active_server: State<'_, crate::server::ActiveServer>,
    new_server: Server,
) -> Result<Vec<Server>, String> {
    let body = serde_json::to_value(&new_server).map_err(|e| e.to_string())?;
    crate::client::remote_api::execute_mutation(&app, "servers", reqwest::Method::POST, Some(&body), || {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        insert_server(&conn, &new_server)?;
        active_server.broadcast(crate::server::types::ServerBroadcastEvent::EntitiesUpdated);
        get_servers(&conn)
    })
    .await
}

#[tauri::command]
pub async fn update_server(
    app: AppHandle,
    db: State<'_, DbState>,
    active_server: State<'_, crate::server::ActiveServer>,
    id: String,
    updated_server: Server,
) -> Result<Vec<Server>, String> {
    let body = serde_json::to_value(&updated_server).map_err(|e| e.to_string())?;
    crate::client::remote_api::execute_mutation(
        &app,
        &format!("servers/{id}"),
        reqwest::Method::PUT,
        Some(&body),
        || {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            update_server_entry(&conn, &id, &updated_server)?;
            active_server.broadcast(crate::server::types::ServerBroadcastEvent::EntitiesUpdated);
            get_servers(&conn)
        },
    )
    .await
}

#[tauri::command]
pub async fn delete_server(
    app: AppHandle,
    db: State<'_, DbState>,
    active_server: State<'_, crate::server::ActiveServer>,
    id: String,
) -> Result<Vec<Server>, String> {
    crate::client::remote_api::execute_mutation(
        &app,
        &format!("servers/{id}"),
        reqwest::Method::DELETE,
        None,
        || {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            delete_server_entry(&conn, &id)?;
            active_server.broadcast(crate::server::types::ServerBroadcastEvent::EntitiesUpdated);
            get_servers(&conn)
        },
    )
    .await
}

#[tauri::command]
pub async fn reorder_servers(
    app: AppHandle,
    db: State<'_, DbState>,
    active_server: State<'_, crate::server::ActiveServer>,
    servers: Vec<Server>,
) -> Result<Vec<Server>, String> {
    let body = serde_json::to_value(&servers).map_err(|e| e.to_string())?;
    crate::client::remote_api::execute_mutation(
        &app,
        "servers/reorder",
        reqwest::Method::POST,
        Some(&body),
        || {
            let mut conn = db.0.lock().map_err(|e| e.to_string())?;
            update_sort_order(&mut conn, &servers)?;
            active_server.broadcast(crate::server::types::ServerBroadcastEvent::EntitiesUpdated);
            get_servers(&conn)
        },
    )
    .await
}

#[tauri::command]
pub fn get_server_logs(
    db: State<'_, DbState>,
    server_id: String,
    limit: Option<i64>,
) -> Result<Vec<ServerLog>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    query_logs(&conn, &server_id, limit.unwrap_or(DEFAULT_LOG_LIMIT))
}
