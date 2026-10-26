use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const APP_DIR_NAME: &str = "caffeine";
const SETTINGS_FILE_NAME: &str = "settings.json";
pub const DEFAULT_TARGET_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 6310;
pub const DEFAULT_RETENTION_DAYS: u32 = 14;
pub const DEFAULT_MAX_LOG_ENTRIES: u32 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    #[serde(default = "default_target_host")]
    pub target_host: String,
    #[serde(default = "default_port")]
    pub target_port: u16,
    #[serde(default)]
    pub server_mode: bool,
    #[serde(default = "default_port")]
    pub server_port: u16,
    #[serde(default = "default_server_bind")]
    pub server_bind: String,
    #[serde(default)]
    pub admin_token: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    #[serde(default = "default_max_log_entries")]
    pub max_log_entries: u32,
}

fn default_target_host() -> String {
    DEFAULT_TARGET_HOST.to_string()
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_server_bind() -> String {
    "0.0.0.0".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_retention_days() -> u32 {
    DEFAULT_RETENTION_DAYS
}

fn default_max_log_entries() -> u32 {
    DEFAULT_MAX_LOG_ENTRIES
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            target_host: default_target_host(),
            target_port: default_port(),
            server_mode: false,
            server_port: default_port(),
            server_bind: default_server_bind(),
            admin_token: None,
            theme: default_theme(),
            retention_days: default_retention_days(),
            max_log_entries: default_max_log_entries(),
        }
    }
}

pub fn get_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .or_else(|_| {
            app.path()
                .config_dir()
                .map(|p| p.join(APP_DIR_NAME))
        })
        .map_err(|e| format!("Failed to resolve user config directory: {e}"))?;

    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory at {:?}: {e}", config_dir))?;

    Ok(config_dir.join(SETTINGS_FILE_NAME))
}

pub fn load_app_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let path = get_settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings file at {:?}: {e}", path))?;

    serde_json::from_str::<AppSettings>(&contents)
        .map_err(|e| format!("Failed to parse settings JSON at {:?}: {e}", path))
}

pub fn save_app_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path(app)?;
    let serialized = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;

    fs::write(&path, serialized)
        .map_err(|e| format!("Failed to write settings to {:?}: {e}", path))?;

    Ok(())
}

pub fn is_self_serving(host: &str) -> bool {
    let trimmed = host.trim().to_ascii_lowercase();
    trimmed == "127.0.0.1" || trimmed == "localhost" || trimmed == "::1" || trimmed == "0.0.0.0"
}

pub fn sanitize_host_slug(host: &str, port: u16) -> String {
    let stripped = host
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let host_part = stripped.split('/').next().unwrap_or(stripped);
    let clean_host = if let Some(bracket_end) = host_part.find(']') {
        &host_part[..=bracket_end]
    } else {
        host_part.split(':').next().unwrap_or(host_part)
    };

    let sanitized: String = clean_host
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    format!("{sanitized}_{port}")
}

#[tauri::command]
pub fn get_app_settings_cmd(app: AppHandle) -> Result<AppSettings, String> {
    load_app_settings(&app)
}

#[tauri::command]
pub async fn update_app_settings_cmd(
    app: AppHandle,
    db_state: tauri::State<'_, crate::db::DbState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let old_settings = load_app_settings(&app).unwrap_or_default();
    save_app_settings(&app, &settings)?;

    if old_settings.server_mode != settings.server_mode
        || old_settings.target_host != settings.target_host
        || old_settings.target_port != settings.target_port
    {
        crate::db::switch_active_db(&app, &db_state)?;

        if settings.server_mode && !old_settings.server_mode {
            if let Ok(machine_db) = crate::db::get_machine_db_path() {
                let bind = settings.server_bind.clone();
                let port = settings.server_port;
                tokio::spawn(async move {
                    if let Err(e) = crate::server::start_embedded_server(&bind, port, machine_db).await {
                        eprintln!("Failed to start embedded server on settings change: {e}");
                    }
                });
            }
        }
    }

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_self_serving() {
        assert!(is_self_serving("127.0.0.1"));
        assert!(is_self_serving("localhost"));
        assert!(is_self_serving("LOCALHOST"));
        assert!(is_self_serving("::1"));
        assert!(!is_self_serving("192.168.1.50"));
        assert!(!is_self_serving("monitor.corp.lan"));
    }

    #[test]
    fn test_sanitize_host_slug() {
        assert_eq!(sanitize_host_slug("192.168.1.50", 6310), "192.168.1.50_6310");
        assert_eq!(sanitize_host_slug("http://192.168.1.50:6310", 6310), "192.168.1.50_6310");
        assert_eq!(sanitize_host_slug("monitor.internal.net", 8080), "monitor.internal.net_8080");
        assert_eq!(sanitize_host_slug("[::1]", 6310), "___1__6310");
        assert_eq!(sanitize_host_slug("[::1]:6310", 6310), "___1__6310");
    }

    #[test]
    fn test_default_settings_match_expectations() {
        let settings = AppSettings::default();
        assert_eq!(settings.target_host, "127.0.0.1");
        assert_eq!(settings.target_port, 6310);
        assert!(!settings.server_mode);
        assert_eq!(settings.server_port, 6310);
        assert_eq!(settings.retention_days, 14);
        assert_eq!(settings.max_log_entries, 1000);
    }
}
