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
pub struct ServerTokenEntry {
    pub id: String,
    pub label: String,
    pub token: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerTokenPublic {
    pub id: String,
    pub label: String,
    pub created_at: String,
    pub token_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratedTokenResponse {
    pub id: String,
    pub label: String,
    pub token: String,
    pub created_at: String,
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
    pub server_tokens: Vec<ServerTokenEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_token: Option<String>,
}

impl Default for MachineSettings {
    fn default() -> Self {
        Self {
            server_mode: false,
            server_port: default_port(),
            server_bind: default_server_bind(),
            server_tokens: Vec::new(),
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
    pub server_tokens: Vec<ServerTokenPublic>,
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
            server_tokens: Vec::new(),
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
    load_machine_settings_from_path(&path)
}

fn load_machine_settings_from_path(path: &std::path::Path) -> Result<MachineSettings, String> {
    if !path.exists() {
        return Ok(MachineSettings::default());
    }

    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read machine settings at {:?}: {e}", path))?;

    let mut settings = serde_json::from_str::<MachineSettings>(&contents)
        .map_err(|e| format!("Failed to parse machine settings JSON at {:?}: {e}", path))?;

    if settings.server_tokens.is_empty() {
        if let Some(tok) = &settings.admin_token {
            if !tok.trim().is_empty() {
                settings.server_tokens.push(ServerTokenEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    label: "Default Admin".to_string(),
                    token: tok.trim().to_string(),
                    created_at: current_utc_iso(),
                });
            }
        }
    }

    Ok(settings)
}

pub fn save_machine_settings(settings: &MachineSettings) -> Result<(), String> {
    let machine_dir = crate::db::ensure_machine_db_dir()?;
    let path = machine_dir.join(MACHINE_SETTINGS_FILE_NAME);
    let serialized = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize machine settings: {e}"))?;

    fs::write(&path, serialized).map_err(|e| {
        format!(
            "Failed to write machine settings to {:?}: {e}. Elevated administrator privileges may be required to modify machine-wide server settings.",
            path
        )
    })?;

    Ok(())
}

pub fn load_app_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let user_path = get_settings_path(app)?;
    let user_settings: UserProfileSettings = if user_path.exists() {
        let contents = fs::read_to_string(&user_path)
            .map_err(|e| format!("Failed to read settings file at {:?}: {e}", user_path))?;
        serde_json::from_str::<UserProfileSettings>(&contents)
            .map_err(|e| format!("Failed to parse user settings JSON at {:?}: {e}", user_path))?
    } else {
        UserProfileSettings::default()
    };

    let machine_settings = load_machine_settings()?;
    let public_tokens = machine_settings
        .server_tokens
        .iter()
        .map(|e| ServerTokenPublic {
            id: e.id.clone(),
            label: e.label.clone(),
            created_at: e.created_at.clone(),
            token_preview: mask_token(&e.token),
        })
        .collect();

    Ok(AppSettings {
        target_host: user_settings.target_host,
        target_port: user_settings.target_port,
        server_mode: machine_settings.server_mode,
        server_port: machine_settings.server_port,
        server_bind: machine_settings.server_bind,
        server_tokens: public_tokens,
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

    let current_machine_settings = load_machine_settings()?;
    let new_machine_settings = MachineSettings {
        server_mode: settings.server_mode,
        server_port: settings.server_port,
        server_bind: settings.server_bind.clone(),
        server_tokens: current_machine_settings.server_tokens.clone(),
        admin_token: None,
    };

    if new_machine_settings != current_machine_settings {
        save_machine_settings(&new_machine_settings)?;
    }

    Ok(())
}

pub fn is_self_serving(host: &str) -> bool {
    let trimmed = host.trim().to_ascii_lowercase();
    trimmed == "127.0.0.1" || trimmed == "localhost" || trimmed == "::1" || trimmed == "0.0.0.0"
}

pub fn build_http_base_url(host: &str, port: u16) -> String {
    format!("http://{host}:{port}")
}

pub fn build_ws_url(host: &str, port: u16, path: &str) -> String {
    let clean_path = path.trim_start_matches('/');
    format!("ws://{host}:{port}/{clean_path}")
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
    let old_settings = load_app_settings(&app)?;
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

pub fn mask_token(token: &str) -> String {
    let suffix = if token.len() >= 4 {
        &token[token.len() - 4..]
    } else {
        token
    };
    format!("caff_••••••••{suffix}")
}

pub fn generate_secure_token() -> String {
    let mut bytes = [0u8; 32];
    argon2::password_hash::rand_core::RngCore::fill_bytes(
        &mut argon2::password_hash::rand_core::OsRng,
        &mut bytes,
    );
    format!("caff_sec_{}", hex::encode(bytes))
}

pub fn current_utc_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_epoch_secs_as_iso(secs)
}

pub fn format_epoch_secs_as_iso(secs: u64) -> String {
    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let minutes = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    let mut year = 1970;
    let mut d = days;

    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        year += 1;
    }

    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_months = [
        31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    let mut month = 1;
    for &dim in &days_in_months {
        if d < dim {
            break;
        }
        d -= dim;
        month += 1;
    }
    let day = d + 1;

    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

pub fn add_server_token(label: &str) -> Result<GeneratedTokenResponse, String> {
    let trimmed_label = label.trim();
    if trimmed_label.is_empty() {
        return Err("Token label cannot be empty".to_string());
    }

    let mut machine = load_machine_settings()?;
    if machine
        .server_tokens
        .iter()
        .any(|t| t.label.eq_ignore_ascii_case(trimmed_label))
    {
        return Err(format!("A token with the label '{trimmed_label}' already exists"));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let token = generate_secure_token();
    let created_at = current_utc_iso();

    let entry = ServerTokenEntry {
        id: id.clone(),
        label: trimmed_label.to_string(),
        token: token.clone(),
        created_at: created_at.clone(),
    };

    machine.server_tokens.push(entry);
    save_machine_settings(&machine)?;

    Ok(GeneratedTokenResponse {
        id,
        label: trimmed_label.to_string(),
        token,
        created_at,
    })
}

pub fn regenerate_server_token(id: &str) -> Result<GeneratedTokenResponse, String> {
    let mut machine = load_machine_settings()?;
    let entry = machine
        .server_tokens
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| format!("Token with ID '{id}' not found"))?;

    let new_token = generate_secure_token();
    let new_created_at = current_utc_iso();

    entry.token = new_token.clone();
    entry.created_at = new_created_at.clone();
    let label = entry.label.clone();
    let token_id = entry.id.clone();

    save_machine_settings(&machine)?;

    Ok(GeneratedTokenResponse {
        id: token_id,
        label,
        token: new_token,
        created_at: new_created_at,
    })
}

pub fn delete_server_token(id: &str) -> Result<(), String> {
    let mut machine = load_machine_settings()?;
    let original_len = machine.server_tokens.len();
    machine.server_tokens.retain(|t| t.id != id);

    if machine.server_tokens.len() == original_len {
        return Err(format!("Token with ID '{id}' not found"));
    }

    save_machine_settings(&machine)
}

pub fn list_server_tokens() -> Result<Vec<ServerTokenPublic>, String> {
    let machine = load_machine_settings()?;
    Ok(machine
        .server_tokens
        .into_iter()
        .map(|t| ServerTokenPublic {
            id: t.id,
            label: t.label,
            created_at: t.created_at,
            token_preview: mask_token(&t.token),
        })
        .collect())
}

#[tauri::command]
pub fn get_server_tokens_cmd() -> Result<Vec<ServerTokenPublic>, String> {
    list_server_tokens()
}

#[tauri::command]
pub fn create_server_token_cmd(label: String) -> Result<GeneratedTokenResponse, String> {
    add_server_token(&label)
}

#[tauri::command]
pub fn regenerate_server_token_cmd(id: String) -> Result<GeneratedTokenResponse, String> {
    regenerate_server_token(&id)
}

#[tauri::command]
pub fn delete_server_token_cmd(id: String) -> Result<(), String> {
    delete_server_token(&id)
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
            server_tokens: Vec::new(),
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
            server_tokens: Vec::new(),
            admin_token: None,
        };
        let ms2 = MachineSettings::default();
        assert_eq!(ms1, ms2);

        let ms3 = MachineSettings {
            server_mode: true,
            server_port: 6310,
            server_bind: "0.0.0.0".to_string(),
            server_tokens: Vec::new(),
            admin_token: None,
        };
        assert_ne!(ms1, ms3);
    }

    #[test]
    fn test_mask_token() {
        assert_eq!(mask_token("caff_sec_abcdef1234"), "caff_••••••••1234");
        assert_eq!(mask_token("1234"), "caff_••••••••1234");
        assert_eq!(mask_token("abc"), "caff_••••••••abc");
    }

    #[test]
    fn test_generate_secure_token() {
        let t1 = generate_secure_token();
        let t2 = generate_secure_token();
        assert!(t1.starts_with("caff_sec_"));
        assert!(t2.starts_with("caff_sec_"));
        assert_ne!(t1, t2);
        assert_eq!(t1.len(), "caff_sec_".len() + 64);
    }

    #[test]
    fn test_format_epoch_secs_as_iso() {
        assert_eq!(format_epoch_secs_as_iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_epoch_secs_as_iso(1730000000), "2024-10-27T03:33:20Z");
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{}_{nanos}", std::process::id()))
    }

    #[test]
    fn test_load_machine_settings_missing_returns_default_without_creating_dir() {
        let non_existent_dir = unique_test_dir("caffeine_config_missing");
        let settings_path = non_existent_dir.join(MACHINE_SETTINGS_FILE_NAME);

        let loaded = load_machine_settings_from_path(&settings_path).unwrap();
        assert_eq!(loaded, MachineSettings::default());
        assert!(!non_existent_dir.exists());
    }

    #[test]
    fn test_load_machine_settings_migrates_legacy_admin_token() {
        let dir = unique_test_dir("caffeine_config_migrate");
        fs::create_dir_all(&dir).unwrap();
        let settings_path = dir.join(MACHINE_SETTINGS_FILE_NAME);
        fs::write(&settings_path, r#"{"server_mode":true,"admin_token":"legacy_tok"}"#).unwrap();

        let loaded = load_machine_settings_from_path(&settings_path).unwrap();
        assert!(loaded.server_mode);
        assert_eq!(loaded.server_tokens.len(), 1);
        assert_eq!(loaded.server_tokens[0].token, "legacy_tok");

        let _ = fs::remove_dir_all(&dir);
    }
}
