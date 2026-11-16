pub mod db;
pub mod notes;
pub mod script_formatter;
pub mod scripts;
pub mod todo;
pub mod types;
pub mod worker;

use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

const WORKSPACE_HOME_DIR_NAME: &str = "Caffeine";

pub fn resolve_configured_workspace_dir(
    app: &AppHandle,
    configured: Option<&str>,
) -> Result<PathBuf, String> {
    let workspace_dir = match configured.map(str::trim).filter(|s| !s.is_empty()) {
        Some(custom) => PathBuf::from(custom),
        None => app
            .path()
            .home_dir()
            .map_err(|e| format!("Failed to resolve user home directory: {e}"))?
            .join(WORKSPACE_HOME_DIR_NAME),
    };

    fs::create_dir_all(&workspace_dir)
        .map_err(|e| format!("Failed to create workspace directory at {workspace_dir:?}: {e}"))?;
    Ok(workspace_dir)
}

pub fn get_user_workspace_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let settings = crate::config::load_app_settings(app)?;
    resolve_configured_workspace_dir(app, settings.workspace_dir.as_deref())
}

pub fn migrate_workspace_directory(old_dir: &Path, new_dir: &Path) -> Result<(), String> {
    if old_dir == new_dir {
        return Ok(());
    }
    if old_dir.exists()
        && new_dir.exists()
        && let (Ok(old_canon), Ok(new_canon)) = (fs::canonicalize(old_dir), fs::canonicalize(new_dir))
        && old_canon == new_canon
    {
        return Ok(());
    }

    fs::create_dir_all(new_dir)
        .map_err(|e| format!("Failed to create new workspace directory {new_dir:?}: {e}"))?;

    if !old_dir.is_dir() {
        return Ok(());
    }

    for subdir in ["Notes", "Scripts"] {
        migrate_legacy_workspace_subdir(&old_dir.join(subdir), &new_dir.join(subdir))?;
    }

    let _ = fs::remove_dir(old_dir);
    Ok(())
}

pub(crate) fn migrate_legacy_workspace_subdir(
    legacy_dir: &Path,
    target_dir: &Path,
) -> Result<(), String> {
    if !legacy_dir.is_dir() {
        return Ok(());
    }

    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create target workspace directory {target_dir:?}: {e}"))?;

    let entries = fs::read_dir(legacy_dir)
        .map_err(|e| format!("Failed to read legacy workspace directory {legacy_dir:?}: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            format!("Failed to inspect entry in legacy workspace directory {legacy_dir:?}: {e}")
        })?;
        let src_path = entry.path();
        if !src_path.is_file() {
            continue;
        }

        let dst_path = target_dir.join(entry.file_name());
        if dst_path.exists() {
            continue;
        }

        if let Err(rename_err) = fs::rename(&src_path, &dst_path) {
            // Fall back to copy + remove when app_config_dir and home_dir live on separate volumes.
            fs::copy(&src_path, &dst_path).map_err(|copy_err| {
                format!(
                    "Failed to migrate workspace file {src_path:?} to {dst_path:?} (rename: {rename_err}; copy: {copy_err})"
                )
            })?;
            fs::remove_file(&src_path).map_err(|remove_err| {
                format!(
                    "Copied workspace file to {dst_path:?} but failed to remove legacy file {src_path:?}: {remove_err}"
                )
            })?;
        }
    }

    let _ = fs::remove_dir(legacy_dir);
    Ok(())
}

pub(crate) fn resolve_user_workspace_root(
    app: &AppHandle,
    legacy_subdir: &str,
    target_subdir: &str,
) -> Result<PathBuf, String> {
    let workspace_dir = get_user_workspace_dir(app)?;
    let target_dir = workspace_dir.join(target_subdir);
    let legacy_dir = crate::config::get_user_roaming_dir(app)?.join(legacy_subdir);
    migrate_legacy_workspace_subdir(&legacy_dir, &target_dir)?;
    Ok(workspace_dir)
}

pub(crate) fn current_time_ms() -> Result<i64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock is before Unix epoch: {e}"))?;
    i64::try_from(duration.as_millis())
        .map_err(|e| format!("System timestamp overflowed i64: {e}"))
}

pub(crate) fn file_modified_ms(metadata: &Metadata) -> Result<i64, String> {
    let modified = metadata
        .modified()
        .map_err(|e| format!("Failed to read file modification timestamp: {e}"))?;
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("File modification time precedes Unix epoch: {e}"))?;
    i64::try_from(duration.as_millis())
        .map_err(|e| format!("File timestamp overflowed i64: {e}"))
}

pub(crate) fn validate_safe_slug(slug: &str, kind: &str) -> Result<(), String> {
    if slug.is_empty() || slug.len() > 128 {
        return Err(format!(
            "Invalid {kind} identifier '{slug}': must be between 1 and 128 characters"
        ));
    }
    if !slug
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(format!(
            "Invalid {kind} identifier '{slug}': only ASCII letters, digits, '-' and '_' are allowed"
        ));
    }
    Ok(())
}

pub(crate) fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in input.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
pub(crate) fn temp_roaming_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{nanos}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_legacy_workspace_subdir_moves_files_and_preserves_conflicts() {
        let root = temp_roaming_dir("caffeine_workspace_migrate");
        let legacy_dir = root.join("legacy_notes");
        let target_dir = root.join("Caffeine").join("Notes");
        fs::create_dir_all(&legacy_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();

        fs::write(legacy_dir.join("runbook.md"), "# Runbook").unwrap();
        fs::write(legacy_dir.join("shared.md"), "# Old").unwrap();
        fs::write(target_dir.join("shared.md"), "# New").unwrap();

        migrate_legacy_workspace_subdir(&legacy_dir, &target_dir).unwrap();

        assert_eq!(
            fs::read_to_string(target_dir.join("runbook.md")).unwrap(),
            "# Runbook"
        );
        assert_eq!(
            fs::read_to_string(target_dir.join("shared.md")).unwrap(),
            "# New"
        );
        assert!(!legacy_dir.join("runbook.md").exists());
        assert!(legacy_dir.join("shared.md").exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_migrate_legacy_workspace_subdir_noops_when_missing() {
        let root = temp_roaming_dir("caffeine_workspace_missing");
        let legacy_dir = root.join("missing");
        let target_dir = root.join("Caffeine").join("Scripts");

        migrate_legacy_workspace_subdir(&legacy_dir, &target_dir).unwrap();
        assert!(!target_dir.exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_migrate_workspace_directory_moves_notes_and_scripts() {
        let root = temp_roaming_dir("caffeine_workspace_relocate");
        let old_dir = root.join("old_caffeine");
        let new_dir = root.join("network_share").join("Caffeine");
        fs::create_dir_all(old_dir.join("Notes")).unwrap();
        fs::create_dir_all(old_dir.join("Scripts")).unwrap();

        fs::write(old_dir.join("Notes").join("dns.md"), "# DNS").unwrap();
        fs::write(old_dir.join("Scripts").join("ping.ps1"), "Test-Connection").unwrap();

        migrate_workspace_directory(&old_dir, &new_dir).unwrap();

        assert_eq!(
            fs::read_to_string(new_dir.join("Notes").join("dns.md")).unwrap(),
            "# DNS"
        );
        assert_eq!(
            fs::read_to_string(new_dir.join("Scripts").join("ping.ps1")).unwrap(),
            "Test-Connection"
        );
        assert!(!old_dir.exists());

        migrate_workspace_directory(&new_dir, &new_dir).unwrap();
        assert!(new_dir.join("Notes").join("dns.md").is_file());

        let _ = fs::remove_dir_all(&root);
    }
}
