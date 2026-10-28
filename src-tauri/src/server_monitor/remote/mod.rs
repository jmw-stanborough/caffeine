pub mod rdp;
pub mod ssh;
pub mod validation;

use tauri::AppHandle;

#[tauri::command]
pub fn launch_remote_desktop(
    host: String,
    port: Option<u16>,
    username: Option<String>,
) -> Result<(), String> {
    rdp::launch_rdp_client(&host, port, username.as_deref())
}

#[tauri::command]
pub fn start_server_ssh_terminal(
    app: AppHandle,
    server_id: String,
    host: String,
    port: Option<u16>,
    username: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    ssh::start_ssh_session(
        app,
        &server_id,
        &host,
        port,
        username.as_deref(),
        cols,
        rows,
    )
}

#[tauri::command]
pub fn write_server_ssh_terminal(server_id: String, data: String) -> Result<(), String> {
    ssh::write_to_ssh_session(&server_id, &data)
}

#[tauri::command]
pub fn resize_server_ssh_terminal(server_id: String, cols: u16, rows: u16) -> Result<(), String> {
    ssh::resize_ssh_session(&server_id, cols, rows)
}

#[tauri::command]
pub fn close_server_ssh_terminal(server_id: String) -> Result<(), String> {
    ssh::close_ssh_session(&server_id);
    Ok(())
}
