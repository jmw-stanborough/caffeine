use super::types::{Server, ServerMonitorSettings};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

fn get_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .config_dir()
        .map_err(|e| format!("Failed to resolve config dir: {e}"))?;

    let settings_dir = config_dir.join("caffeine").join("server-monitor");

    fs::create_dir_all(&settings_dir)
        .map_err(|e| format!("Failed to create settings directory: {e}"))?;

    Ok(settings_dir.join("settings.json"))
}

fn read_settings(path: &Path) -> Result<ServerMonitorSettings, String> {
    if !path.exists() {
        return Ok(ServerMonitorSettings::default());
    }

    let contents =
        fs::read_to_string(path).map_err(|e| format!("Failed to read settings file: {e}"))?;

    if contents.trim().is_empty() {
        return Ok(ServerMonitorSettings::default());
    }

    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse settings JSON: {e}"))
}

fn write_settings(path: &Path, settings: &ServerMonitorSettings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    fs::write(path, json).map_err(|e| format!("Failed to write settings file: {e}"))
}

#[tauri::command]
pub fn load_servers(app: AppHandle) -> Result<Vec<Server>, String> {
    let path = get_settings_path(&app)?;

    if !path.exists() {
        let initial_settings = ServerMonitorSettings::default();
        write_settings(&path, &initial_settings)?;
        return Ok(initial_settings.servers);
    }

    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read settings file: {e}"))?;

    let settings: ServerMonitorSettings = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse settings JSON: {e}"))?;

    if !contents.contains("\"id\"") {
        let _ = write_settings(&path, &settings);
    }

    Ok(settings.servers)
}

#[tauri::command]
pub fn save_server(app: AppHandle, new_server: Server) -> Result<Vec<Server>, String> {
    let path = get_settings_path(&app)?;
    let mut settings = read_settings(&path)?;

    let mut server = new_server;
    if server.id.is_empty() {
        server.id = format!("srv-{}", uuid::Uuid::new_v4());
    }

    settings.servers.push(server);
    write_settings(&path, &settings)?;

    Ok(settings.servers)
}

#[tauri::command]
pub fn update_server(
    app: AppHandle,
    id: String,
    updated_server: Server,
) -> Result<Vec<Server>, String> {
    let path = get_settings_path(&app)?;
    let mut settings = read_settings(&path)?;

    if let Some(target) = settings.servers.iter_mut().find(|s| s.id == id) {
        *target = updated_server;
        target.id = id;
    } else {
        return Err(format!("Server with id '{id}' not found"));
    }

    write_settings(&path, &settings)?;
    Ok(settings.servers)
}

#[tauri::command]
pub fn delete_server(app: AppHandle, id: String) -> Result<Vec<Server>, String> {
    let path = get_settings_path(&app)?;
    let mut settings = read_settings(&path)?;

    settings.servers.retain(|s| s.id != id);

    write_settings(&path, &settings)?;
    Ok(settings.servers)
}

#[tauri::command]
pub fn reorder_servers(app: AppHandle, servers: Vec<Server>) -> Result<(), String> {
    let path = get_settings_path(&app)?;
    let settings = ServerMonitorSettings { servers };
    write_settings(&path, &settings)
}
