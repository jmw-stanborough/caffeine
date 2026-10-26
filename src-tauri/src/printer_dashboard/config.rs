use super::db::{
    delete_printer_entry, get_printer_filter_tags, get_printers, insert_printer,
    query_printer_logs, save_printer_filter_tags_entries, update_printer_entry,
    update_printer_sort_order,
};
use super::types::{Printer, PrinterFilterConfig, PrinterLog};
use crate::db::{DEFAULT_LOG_LIMIT, DbState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn load_printers(db: State<'_, DbState>) -> Result<Vec<Printer>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    get_printers(&conn)
}

#[tauri::command]
pub async fn save_printer(
    app: AppHandle,
    db: State<'_, DbState>,
    active_server: State<'_, crate::server::ActiveServer>,
    new_printer: Printer,
) -> Result<Vec<Printer>, String> {
    let body = serde_json::to_value(&new_printer).map_err(|e| e.to_string())?;
    crate::client::remote_api::execute_mutation(&app, "printers", reqwest::Method::POST, Some(&body), || {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        insert_printer(&conn, &new_printer)?;
        active_server.broadcast(crate::server::types::ServerBroadcastEvent::EntitiesUpdated);
        get_printers(&conn)
    })
    .await
}

#[tauri::command]
pub async fn update_printer(
    app: AppHandle,
    db: State<'_, DbState>,
    active_server: State<'_, crate::server::ActiveServer>,
    id: String,
    updated_printer: Printer,
) -> Result<Vec<Printer>, String> {
    let body = serde_json::to_value(&updated_printer).map_err(|e| e.to_string())?;
    crate::client::remote_api::execute_mutation(
        &app,
        &format!("printers/{id}"),
        reqwest::Method::PUT,
        Some(&body),
        || {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            update_printer_entry(&conn, &id, &updated_printer)?;
            active_server.broadcast(crate::server::types::ServerBroadcastEvent::EntitiesUpdated);
            get_printers(&conn)
        },
    )
    .await
}

#[tauri::command]
pub async fn delete_printer(
    app: AppHandle,
    db: State<'_, DbState>,
    active_server: State<'_, crate::server::ActiveServer>,
    id: String,
) -> Result<Vec<Printer>, String> {
    crate::client::remote_api::execute_mutation(
        &app,
        &format!("printers/{id}"),
        reqwest::Method::DELETE,
        None,
        || {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            delete_printer_entry(&conn, &id)?;
            active_server.broadcast(crate::server::types::ServerBroadcastEvent::EntitiesUpdated);
            get_printers(&conn)
        },
    )
    .await
}

#[tauri::command]
pub async fn reorder_printers(
    app: AppHandle,
    db: State<'_, DbState>,
    active_server: State<'_, crate::server::ActiveServer>,
    printers: Vec<Printer>,
) -> Result<Vec<Printer>, String> {
    let body = serde_json::to_value(&printers).map_err(|e| e.to_string())?;
    crate::client::remote_api::execute_mutation(
        &app,
        "printers/reorder",
        reqwest::Method::POST,
        Some(&body),
        || {
            let mut conn = db.0.lock().map_err(|e| e.to_string())?;
            update_printer_sort_order(&mut conn, &printers)?;
            active_server.broadcast(crate::server::types::ServerBroadcastEvent::EntitiesUpdated);
            get_printers(&conn)
        },
    )
    .await
}

#[tauri::command]
pub fn get_printer_logs(
    db: State<'_, DbState>,
    printer_id: String,
    limit: Option<i64>,
) -> Result<Vec<PrinterLog>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    query_printer_logs(&conn, &printer_id, limit.unwrap_or(DEFAULT_LOG_LIMIT))
}

#[tauri::command]
pub fn load_printer_filter_tags(db: State<'_, DbState>) -> Result<PrinterFilterConfig, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    get_printer_filter_tags(&conn)
}

#[tauri::command]
pub fn save_printer_filter_tags(
    db: State<'_, DbState>,
    all_tags: Vec<String>,
    any_tags: Vec<String>,
) -> Result<PrinterFilterConfig, String> {
    let mut conn = db.0.lock().map_err(|e| e.to_string())?;
    let config = PrinterFilterConfig { all_tags, any_tags };
    save_printer_filter_tags_entries(&mut conn, &config)?;
    get_printer_filter_tags(&conn)
}
