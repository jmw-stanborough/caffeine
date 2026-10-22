use super::db::{
    delete_server_entry, get_servers, insert_server, query_logs, update_server_entry,
    update_sort_order,
};
use super::types::{Server, ServerLog};
use crate::db::{DEFAULT_LOG_LIMIT, DbState};
use tauri::State;

#[tauri::command]
pub fn load_servers(db: State<'_, DbState>) -> Result<Vec<Server>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    get_servers(&conn)
}

#[tauri::command]
pub fn save_server(db: State<'_, DbState>, new_server: Server) -> Result<Vec<Server>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    insert_server(&conn, &new_server)?;
    get_servers(&conn)
}

#[tauri::command]
pub fn update_server(
    db: State<'_, DbState>,
    id: String,
    updated_server: Server,
) -> Result<Vec<Server>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    update_server_entry(&conn, &id, &updated_server)?;
    get_servers(&conn)
}

#[tauri::command]
pub fn delete_server(db: State<'_, DbState>, id: String) -> Result<Vec<Server>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    delete_server_entry(&conn, &id)?;
    get_servers(&conn)
}

#[tauri::command]
pub fn reorder_servers(
    db: State<'_, DbState>,
    servers: Vec<Server>,
) -> Result<Vec<Server>, String> {
    let mut conn = db.0.lock().map_err(|e| e.to_string())?;
    update_sort_order(&mut conn, &servers)?;
    get_servers(&conn)
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
