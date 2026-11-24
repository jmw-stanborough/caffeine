use std::path::{Path, PathBuf};

use crate::server_monitor::types::HealthStatus;

#[cfg(target_os = "windows")]
mod windows;

pub const APP_AUMID: &str = "uk.jmwalters.caffeine";
const NOTIFICATION_ICON_BYTES: &[u8] = include_bytes!("../../icons/128x128.png");
const NOTIFICATION_ICON_FILENAME: &str = "notification-icon.png";

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const DEFAULT_SERVER_UP_BYTES: &[u8] = include_bytes!("../../../static/sound-files/servers/up.mp3");
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const DEFAULT_SERVER_DOWN_BYTES: &[u8] =
    include_bytes!("../../../static/sound-files/servers/down.mp3");
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const DEFAULT_NOTIFICATION_BYTES: &[u8] =
    include_bytes!("../../../static/sound-files/notification.mp3");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEffect {
    ServerUp,
    ServerDown,
    Notification,
}

impl SoundEffect {
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    fn default_filename_and_bytes(self) -> (&'static str, &'static [u8]) {
        match self {
            SoundEffect::ServerUp => ("server-up.mp3", DEFAULT_SERVER_UP_BYTES),
            SoundEffect::ServerDown => ("server-down.mp3", DEFAULT_SERVER_DOWN_BYTES),
            SoundEffect::Notification => ("notification.mp3", DEFAULT_NOTIFICATION_BYTES),
        }
    }
}

pub fn server_transition_sound(
    previous: &HealthStatus,
    current: &HealthStatus,
) -> Option<SoundEffect> {
    match (previous, current) {
        (HealthStatus::Online, HealthStatus::Offline) => Some(SoundEffect::ServerDown),
        (HealthStatus::Offline, HealthStatus::Online) => Some(SoundEffect::ServerUp),
        _ => None,
    }
}

pub fn is_native_sound_playback_enabled() -> bool {
    cfg!(target_os = "windows")
}

#[tauri::command]
pub fn is_native_sound_playback_enabled_cmd() -> bool {
    is_native_sound_playback_enabled()
}

pub fn play_sound_effect(app: &tauri::AppHandle, effect: SoundEffect) {
    #[cfg(target_os = "windows")]
    {
        let custom_path = match crate::config::load_app_settings(app) {
            Ok(settings) => match effect {
                SoundEffect::ServerUp => settings.sound_server_up,
                SoundEffect::ServerDown => settings.sound_server_down,
                SoundEffect::Notification => settings.sound_notification,
            },
            Err(err) => {
                eprintln!("Failed to load app settings for native sound playback: {err}");
                None
            }
        };

        let base_dir = match crate::config::get_user_roaming_dir(app) {
            Ok(dir) => dir,
            Err(err) => {
                eprintln!("Failed to resolve roaming directory for native sound playback: {err}");
                std::env::temp_dir().join("caffeine")
            }
        };

        match resolve_effect_path_in_dir(&base_dir, effect, custom_path.as_deref()) {
            Ok(path) => {
                if let Err(err) = windows::play_file(path) {
                    eprintln!("Native sound playback failed for {effect:?}: {err}");
                }
            }
            Err(err) => {
                eprintln!("Failed to prepare native sound file for {effect:?}: {err}");
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, effect);
    }
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) fn resolve_effect_path_in_dir(
    dir: &Path,
    effect: SoundEffect,
    custom_path: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(trimmed) = custom_path.map(str::trim).filter(|s| !s.is_empty()) {
        let candidate = PathBuf::from(trimmed);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    let sounds_dir = dir.join("sounds");
    std::fs::create_dir_all(&sounds_dir)
        .map_err(|e| format!("Failed to create sounds directory {sounds_dir:?}: {e}"))?;

    let (filename, bytes) = effect.default_filename_and_bytes();
    let target_path = sounds_dir.join(filename);
    let needs_write = match std::fs::metadata(&target_path) {
        Ok(meta) => !meta.is_file() || meta.len() != bytes.len() as u64,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(&target_path, bytes)
            .map_err(|e| format!("Failed to write default sound file {target_path:?}: {e}"))?;
    }

    Ok(target_path)
}

pub fn ensure_notification_icon_file(dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("Failed to create notification icon directory {dir:?}: {e}"))?;
    let icon_path = dir.join(NOTIFICATION_ICON_FILENAME);
    std::fs::write(&icon_path, NOTIFICATION_ICON_BYTES)
        .map_err(|e| format!("Failed to write notification icon at {icon_path:?}: {e}"))?;
    Ok(icon_path)
}

pub fn resolve_notification_icon_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let roaming_dir = crate::config::get_user_roaming_dir(app)?;
    ensure_notification_icon_file(&roaming_dir)
}

pub fn ensure_windows_aumid_registered(icon_path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Portable executables lack MSI-installed Start Menu shortcuts, and Windows WinRT reads the toast sender logo from AUMID IconUri rather than per-toast icon fields.
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let subkey_path = format!("Software\\Classes\\AppUserModelId\\{APP_AUMID}");
        let (key, _) = hkcu
            .create_subkey(&subkey_path)
            .map_err(|e| format!("Failed to open or create AUMID registry key '{subkey_path}': {e}"))?;
        key.set_value("DisplayName", &"Caffeine")
            .map_err(|e| format!("Failed to set DisplayName on AUMID registry key '{subkey_path}': {e}"))?;
        let icon_uri = icon_path.to_string_lossy();
        key.set_value("IconUri", &icon_uri.as_ref())
            .map_err(|e| format!("Failed to set IconUri on AUMID registry key '{subkey_path}': {e}"))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = icon_path;
        Ok(())
    }
}

#[tauri::command]
pub fn check_sound_file_cmd(path: String) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    Path::new(trimmed).is_file()
}

#[tauri::command]
pub fn load_custom_sound_cmd(path: String) -> Result<Option<String>, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() || !Path::new(trimmed).is_file() {
        return Ok(None);
    }

    let bytes = std::fs::read(trimmed)
        .map_err(|e| format!("Failed to read custom sound file '{trimmed}': {e}"))?;
    let b64 = encode_base64(&bytes);
    Ok(Some(format!("data:audio/mpeg;base64,{b64}")))
}

fn encode_base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);

    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;

        out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 0x3F) as usize] as char
        } else {
            '='
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{}_{nanos}", std::process::id()))
    }

    #[test]
    fn test_encode_base64_rfc4648_vectors() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
        assert_eq!(encode_base64(b"foob"), "Zm9vYg==");
        assert_eq!(encode_base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(encode_base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_check_sound_file_cmd() {
        assert!(!check_sound_file_cmd(String::new()));
        assert!(!check_sound_file_cmd("   ".to_string()));

        let dir = unique_test_dir("caffeine_sound_check");
        fs::create_dir_all(&dir).unwrap();
        assert!(!check_sound_file_cmd(dir.to_string_lossy().into_owned()));

        let missing = dir.join("missing.mp3");
        assert!(!check_sound_file_cmd(missing.to_string_lossy().into_owned()));

        let valid = dir.join("chime.mp3");
        fs::write(&valid, b"ID3").unwrap();
        assert!(check_sound_file_cmd(format!("  {}  ", valid.to_string_lossy())));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_custom_sound_cmd() {
        assert_eq!(load_custom_sound_cmd(String::new()).unwrap(), None);
        assert_eq!(load_custom_sound_cmd("  \t ".to_string()).unwrap(), None);

        let dir = unique_test_dir("caffeine_sound_load");
        fs::create_dir_all(&dir).unwrap();

        let missing = dir.join("missing.mp3");
        assert_eq!(
            load_custom_sound_cmd(missing.to_string_lossy().into_owned()).unwrap(),
            None
        );

        let valid = dir.join("alert.mp3");
        fs::write(&valid, b"foobar").unwrap();
        assert_eq!(
            load_custom_sound_cmd(format!("  {} ", valid.to_string_lossy())).unwrap(),
            Some("data:audio/mpeg;base64,Zm9vYmFy".to_string())
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ensure_notification_icon_file() {
        let dir = unique_test_dir("caffeine_notification_icon");
        let icon_path = ensure_notification_icon_file(&dir).unwrap();
        assert_eq!(icon_path, dir.join(NOTIFICATION_ICON_FILENAME));
        let bytes = fs::read(&icon_path).unwrap();
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ensure_windows_aumid_registered() {
        let dir = unique_test_dir("caffeine_aumid_icon");
        let icon_path = ensure_notification_icon_file(&dir).unwrap();
        assert_eq!(ensure_windows_aumid_registered(&icon_path), Ok(()));

        #[cfg(target_os = "windows")]
        {
            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
            let subkey_path = format!("Software\\Classes\\AppUserModelId\\{APP_AUMID}");
            let key = hkcu.open_subkey(&subkey_path).unwrap();
            let display_name: String = key.get_value("DisplayName").unwrap();
            let icon_uri: String = key.get_value("IconUri").unwrap();
            assert_eq!(display_name, "Caffeine");
            assert_eq!(icon_uri, icon_path.to_string_lossy().as_ref());
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_server_transition_sound() {
        assert_eq!(
            server_transition_sound(&HealthStatus::Online, &HealthStatus::Offline),
            Some(SoundEffect::ServerDown)
        );
        assert_eq!(
            server_transition_sound(&HealthStatus::Offline, &HealthStatus::Online),
            Some(SoundEffect::ServerUp)
        );

        for (prev, next) in [
            (HealthStatus::Online, HealthStatus::Online),
            (HealthStatus::Online, HealthStatus::Degraded),
            (HealthStatus::Degraded, HealthStatus::Online),
            (HealthStatus::Degraded, HealthStatus::Offline),
            (HealthStatus::Degraded, HealthStatus::Degraded),
            (HealthStatus::Offline, HealthStatus::Offline),
            (HealthStatus::Offline, HealthStatus::Degraded),
        ] {
            assert_eq!(
                server_transition_sound(&prev, &next),
                None,
                "unexpected sound for {prev:?} -> {next:?}"
            );
        }
    }

    #[test]
    fn test_resolve_effect_path_custom_and_default_materialization() {
        let dir = unique_test_dir("caffeine_sound_resolve");
        fs::create_dir_all(&dir).unwrap();

        let custom = dir.join("custom-up.mp3");
        fs::write(&custom, b"custom-audio").unwrap();

        let resolved_custom = resolve_effect_path_in_dir(
            &dir,
            SoundEffect::ServerUp,
            Some(&format!("  {}  ", custom.display())),
        )
        .unwrap();
        assert_eq!(resolved_custom, custom);

        let missing = dir.join("does-not-exist.mp3");
        let missing_str = missing.to_string_lossy().into_owned();
        let resolved_dir_custom = resolve_effect_path_in_dir(
            &dir,
            SoundEffect::ServerDown,
            Some(&dir.to_string_lossy()),
        )
        .unwrap();
        assert_eq!(resolved_dir_custom, dir.join("sounds").join("server-down.mp3"));
        assert_eq!(fs::read(&resolved_dir_custom).unwrap(), DEFAULT_SERVER_DOWN_BYTES);

        let resolved_blank = resolve_effect_path_in_dir(&dir, SoundEffect::Notification, Some("   "))
            .unwrap();
        assert_eq!(resolved_blank, dir.join("sounds").join("notification.mp3"));
        assert_eq!(fs::read(&resolved_blank).unwrap(), DEFAULT_NOTIFICATION_BYTES);

        let resolved_missing =
            resolve_effect_path_in_dir(&dir, SoundEffect::ServerUp, Some(&missing_str)).unwrap();
        assert_eq!(resolved_missing, dir.join("sounds").join("server-up.mp3"));
        assert_eq!(fs::read(&resolved_missing).unwrap(), DEFAULT_SERVER_UP_BYTES);

        // Corrupted length triggers re-materialization of the embedded default MP3.
        fs::write(&resolved_missing, b"truncated").unwrap();
        let rematerialized = resolve_effect_path_in_dir(&dir, SoundEffect::ServerUp, None).unwrap();
        assert_eq!(fs::read(&rematerialized).unwrap(), DEFAULT_SERVER_UP_BYTES);

        let _ = fs::remove_dir_all(&dir);
    }
}
