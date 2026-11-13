use std::path::Path;

pub const APP_AUMID: &str = "uk.jmwalters.caffeine";

pub fn ensure_windows_aumid_registered() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Portable executables lack MSI-installed Start Menu shortcuts, so Windows Toast notifications require an HKCU AUMID entry.
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let subkey_path = format!("Software\\Classes\\AppUserModelId\\{APP_AUMID}");
        let (key, _) = hkcu
            .create_subkey(&subkey_path)
            .map_err(|e| format!("Failed to open or create AUMID registry key '{subkey_path}': {e}"))?;
        key.set_value("DisplayName", &"Caffeine")
            .map_err(|e| format!("Failed to set DisplayName on AUMID registry key '{subkey_path}': {e}"))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
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
    fn test_ensure_windows_aumid_registered() {
        assert_eq!(ensure_windows_aumid_registered(), Ok(()));
    }
}
