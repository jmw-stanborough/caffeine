use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const APP_DIR_NAME: &str = "caffeine";
const SETTINGS_FILE_NAME: &str = "settings.json";
const MACHINE_SETTINGS_FILE_NAME: &str = "machine.json";
pub const DEFAULT_TARGET_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 6310;
pub const DEFAULT_RETENTION_DAYS: u32 = 14;
pub const DEFAULT_MAX_LOG_ENTRIES: u32 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfileSettings {
    #[serde(default = "default_target_host")]
    pub target_host: String,
    #[serde(default = "default_port")]
    pub target_port: u16,
    #[serde(default)]
    pub admin_token: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    #[serde(default = "default_max_log_entries")]
    pub max_log_entries: u32,
}

impl Default for UserProfileSettings {
    fn default() -> Self {
        Self {
            target_host: default_target_host(),
            target_port: default_port(),
            admin_token: None,
            theme: default_theme(),
            retention_days: default_retention_days(),
            max_log_entries: default_max_log_entries(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineSettings {
    #[serde(default)]
    pub server_mode: bool,
    #[serde(default = "default_port")]
    pub server_port: u16,
    #[serde(default = "default_server_bind")]
    pub server_bind: String,
    #[serde(default)]
    pub admin_token: Option<String>,
}

impl Default for MachineSettings {
    fn default() -> Self {
        Self {
            server_mode: false,
            server_port: default_port(),
            server_bind: default_server_bind(),
            admin_token: None,
        }
    }
}

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
    pub server_admin_token: Option<String>,
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
            server_admin_token: None,
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

pub fn get_machine_settings_path() -> Result<PathBuf, String> {
    let machine_dir = crate::db::get_machine_db_dir()?;
    Ok(machine_dir.join(MACHINE_SETTINGS_FILE_NAME))
}

pub fn load_machine_settings() -> Result<MachineSettings, String> {
    let path = get_machine_settings_path()?;
    if !path.exists() {
        return Ok(MachineSettings::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read machine settings at {:?}: {e}", path))?;

    serde_json::from_str::<MachineSettings>(&contents)
        .map_err(|e| format!("Failed to parse machine settings JSON at {:?}: {e}", path))
}

pub fn save_machine_settings(settings: &MachineSettings) -> Result<(), String> {
    let path = get_machine_settings_path()?;
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create machine settings directory: {e}"))?;
            #[cfg(target_os = "windows")]
            crate::db::ensure_machine_dir_permissions(parent);
        }
    }
    let serialized = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize machine settings: {e}"))?;

    fs::write(&path, serialized).map_err(|e| {
        format!(
            "Failed to write machine settings to {:?}: {e}. Elevated administrator privileges may be required to modify machine-wide server settings.",
            path
        )
    })?;

    #[cfg(target_os = "windows")]
    if let Some(parent) = path.parent() {
        crate::db::ensure_machine_dir_permissions(parent);
    }

    Ok(())
}

pub fn load_app_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let user_path = get_settings_path(app)?;
    let user_settings: UserProfileSettings = if user_path.exists() {
        let contents = fs::read_to_string(&user_path)
            .map_err(|e| format!("Failed to read settings file at {:?}: {e}", user_path))?;
        serde_json::from_str::<UserProfileSettings>(&contents)
            .unwrap_or_default()
    } else {
        UserProfileSettings::default()
    };

    let machine_settings = load_machine_settings().unwrap_or_default();

    Ok(AppSettings {
        target_host: user_settings.target_host,
        target_port: user_settings.target_port,
        server_mode: machine_settings.server_mode,
        server_port: machine_settings.server_port,
        server_bind: machine_settings.server_bind,
        server_admin_token: machine_settings.admin_token,
        admin_token: user_settings.admin_token,
        theme: user_settings.theme,
        retention_days: user_settings.retention_days,
        max_log_entries: user_settings.max_log_entries,
    })
}

pub fn save_app_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let user_settings = UserProfileSettings {
        target_host: settings.target_host.clone(),
        target_port: settings.target_port,
        admin_token: settings.admin_token.clone(),
        theme: settings.theme.clone(),
        retention_days: settings.retention_days,
        max_log_entries: settings.max_log_entries,
    };

    let user_path = get_settings_path(app)?;
    let user_serialized = serde_json::to_string_pretty(&user_settings)
        .map_err(|e| format!("Failed to serialize user settings: {e}"))?;
    fs::write(&user_path, user_serialized)
        .map_err(|e| format!("Failed to write user settings to {:?}: {e}", user_path))?;

    let new_machine_settings = MachineSettings {
        server_mode: settings.server_mode,
        server_port: settings.server_port,
        server_bind: settings.server_bind.clone(),
        admin_token: settings.server_admin_token.clone(),
    };

    let current_machine_settings = load_machine_settings().unwrap_or_default();
    if new_machine_settings != current_machine_settings {
        save_machine_settings(&new_machine_settings)?;
    }

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
    active_server: tauri::State<'_, crate::server::ActiveServer>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let old_settings = load_app_settings(&app).unwrap_or_default();
    save_app_settings(&app, &settings)?;

    if old_settings.server_mode != settings.server_mode
        || old_settings.target_host != settings.target_host
        || old_settings.target_port != settings.target_port
    {
        crate::db::switch_active_db(&app, &db_state)?;
    }

    if settings.server_mode {
        if !old_settings.server_mode
            || old_settings.server_port != settings.server_port
            || old_settings.server_bind != settings.server_bind
        {
            let machine_db = crate::db::get_machine_db_path()?;
            active_server
                .start(Some(app.clone()), &settings.server_bind, settings.server_port, machine_db)
                .await?;
        }
    } else if old_settings.server_mode {
        active_server.stop()?;
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

    #[test]
    fn test_user_and_machine_settings_separation() {
        let user = UserProfileSettings {
            target_host: "10.0.0.1".to_string(),
            target_port: 6310,
            admin_token: Some("secret".to_string()),
            theme: "dark".to_string(),
            retention_days: 7,
            max_log_entries: 500,
        };
        let machine = MachineSettings {
            server_mode: true,
            server_port: 6310,
            server_bind: "127.0.0.1".to_string(),
            admin_token: Some("machine_server_token".to_string()),
        };

        let user_json = serde_json::to_string(&user).unwrap();
        let machine_json = serde_json::to_string(&machine).unwrap();

        assert!(!user_json.contains("server_mode"));
        assert!(!machine_json.contains("target_host"));
        assert!(!machine_json.contains("theme"));
        assert!(machine_json.contains("machine_server_token"));
    }

    #[test]
    fn test_machine_settings_equality() {
        let ms1 = MachineSettings {
            server_mode: false,
            server_port: 6310,
            server_bind: "0.0.0.0".to_string(),
            admin_token: None,
        };
        let ms2 = MachineSettings::default();
        assert_eq!(ms1, ms2);

        let ms3 = MachineSettings {
            server_mode: true,
            server_port: 6310,
            server_bind: "0.0.0.0".to_string(),
            admin_token: None,
        };
        assert_ne!(ms1, ms3);
    }
}
