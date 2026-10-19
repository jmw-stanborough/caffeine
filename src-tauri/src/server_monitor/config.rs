use super::types::{Server, ServerMonitorSettings};
use std::fs;
use std::path::PathBuf;
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

#[tauri::command]
pub fn load_servers(app: AppHandle) -> Result<Vec<Server>, String> {
    let path = get_settings_path(&app)?;

    if !path.exists() {
        let initial_settings = ServerMonitorSettings::default();
        let json = serde_json::to_string_pretty(&initial_settings)
            .map_err(|e| format!("Failed to serialize default settings: {e}"))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write default settings: {e}"))?;
        return Ok(initial_settings.servers);
    }

    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let settings: ServerMonitorSettings = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;

    Ok(settings.servers)
}

#[tauri::command]
pub fn save_server(app: AppHandle, new_server: Server) -> Result<Vec<Server>, String> {
    let path = get_settings_path(&app)?;

    let mut settings = if path.exists() {
        let contents =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read settings file: {e}"))?;
        serde_json::from_str::<ServerMonitorSettings>(&contents)
            .map_err(|e| format!("Failed to parse settings JSON: {e}"))?
    } else {
        ServerMonitorSettings::default()
    };

    settings.servers.push(new_server);

    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write settings file: {e}"))?;

    Ok(settings.servers)
}

#[tauri::command]
pub fn update_server(
    app: AppHandle,
    original_host: String,
    updated_server: Server,
) -> Result<Vec<Server>, String> {
    let path = get_settings_path(&app)?;
    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {e}"))?;

    let mut settings: ServerMonitorSettings =
        serde_json::from_str(&contents).map_err(|e| format!("Failed to parse JSON: {e}"))?;

    if let Some(target) = settings
        .servers
        .iter_mut()
        .find(|s| s.host == original_host)
    {
        *target = updated_server;
    } else {
        return Err(format!("Server with host '{original_host}' not found"));
    }

    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {e}"))?;

    Ok(settings.servers)
}
