use super::db::{
    delete_printer_entry, get_printer_filter_tags, get_printers, insert_printer,
    query_printer_logs, save_printer_filter_tags_entries, update_printer_entry,
    update_printer_sort_order,
};
use super::types::{Printer, PrinterFilterConfig, PrinterLog};
use crate::server_monitor::db::DbState;
use tauri::State;

#[tauri::command]
pub fn load_printers(db: State<'_, DbState>) -> Result<Vec<Printer>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    get_printers(&conn)
}

#[tauri::command]
pub fn save_printer(db: State<'_, DbState>, new_printer: Printer) -> Result<Vec<Printer>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    insert_printer(&conn, &new_printer)?;
    get_printers(&conn)
}

#[tauri::command]
pub fn update_printer(
    db: State<'_, DbState>,
    id: String,
    updated_printer: Printer,
) -> Result<Vec<Printer>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    update_printer_entry(&conn, &id, &updated_printer)?;
    get_printers(&conn)
}

#[tauri::command]
pub fn delete_printer(db: State<'_, DbState>, id: String) -> Result<Vec<Printer>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    delete_printer_entry(&conn, &id)?;
    get_printers(&conn)
}

#[tauri::command]
pub fn reorder_printers(
    db: State<'_, DbState>,
    printers: Vec<Printer>,
) -> Result<Vec<Printer>, String> {
    let mut conn = db.0.lock().map_err(|e| e.to_string())?;
    update_printer_sort_order(&mut conn, &printers)?;
    get_printers(&conn)
}

#[tauri::command]
pub fn get_printer_logs(
    db: State<'_, DbState>,
    printer_id: String,
    limit: Option<i64>,
) -> Result<Vec<PrinterLog>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    query_printer_logs(&conn, &printer_id, limit.unwrap_or(50))
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
