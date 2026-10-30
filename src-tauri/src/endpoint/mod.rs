pub mod filesystem;
pub mod native;
pub mod powershell;
pub mod system;
pub mod terminal;
pub mod types;
pub mod updates;

use tauri::{AppHandle, Emitter};
use types::{
    EndpointMachineInfo, OsSupportInfo, ShellDetectionResult, UpdateCheckResult,
    UpdateOperationResult,
};

#[tauri::command]
pub async fn endpoint_check_os_support() -> Result<OsSupportInfo, String> {
    tokio::task::spawn_blocking(system::check_os_support)
        .await
        .map_err(|e| format!("Failed to check OS support: {e}"))?
}

#[tauri::command]
pub async fn endpoint_get_machine_info() -> Result<EndpointMachineInfo, String> {
    tokio::task::spawn_blocking(system::gather_machine_info)
        .await
        .map_err(|e| format!("Hardware collection task panicked: {e}"))?
}

#[tauri::command]
pub async fn endpoint_check_updates() -> Result<UpdateCheckResult, String> {
    tokio::task::spawn_blocking(updates::check_for_updates)
        .await
        .map_err(|e| format!("Update check task panicked: {e}"))?
}

#[tauri::command]
pub async fn endpoint_download_updates(
    app: AppHandle,
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
) -> Result<UpdateOperationResult, String> {
    tokio::task::spawn_blocking(move || {
        updates::download_updates(target_ids, target_id, |p| {
            let _ = app.emit("endpoint:update-progress", &p);
        })
    })
    .await
    .map_err(|e| format!("Download update task panicked: {e}"))?
}

#[tauri::command]
pub async fn endpoint_install_updates(
    app: AppHandle,
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
) -> Result<UpdateOperationResult, String> {
    tokio::task::spawn_blocking(move || {
        updates::install_updates(target_ids, target_id, |p| {
            let _ = app.emit("endpoint:update-progress", &p);
        })
    })
    .await
    .map_err(|e| format!("Install update task panicked: {e}"))?
}

#[tauri::command]
pub async fn endpoint_restart_system() -> Result<(), String> {
    tokio::task::spawn_blocking(updates::restart_system)
        .await
        .map_err(|e| format!("System restart task panicked: {e}"))?
}

#[tauri::command]
pub async fn endpoint_detect_shell() -> Result<ShellDetectionResult, String> {
    tokio::task::spawn_blocking(terminal::detect_shell)
        .await
        .map_err(|e| format!("Shell detection task panicked: {e}"))?
}

#[tauri::command]
pub fn endpoint_start_terminal(
    app: AppHandle,
    cols: u16,
    rows: u16,
) -> Result<ShellDetectionResult, String> {
    terminal::start_terminal_session(app, cols, rows)
}

#[tauri::command]
pub fn endpoint_write_terminal(data: String) -> Result<(), String> {
    terminal::write_to_terminal(&data)
}

#[tauri::command]
pub fn endpoint_resize_terminal(cols: u16, rows: u16) -> Result<(), String> {
    terminal::resize_terminal(cols, rows)
}

#[tauri::command]
pub fn endpoint_close_terminal() -> Result<(), String> {
    terminal::close_terminal_session();
    Ok(())
}
