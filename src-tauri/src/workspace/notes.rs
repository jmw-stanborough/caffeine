use super::types::{NoteDocument, NoteSummary};
use super::{file_modified_ms, slugify, validate_safe_slug};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

const NOTES_DIR_NAME: &str = "Notes";
const LEGACY_NOTES_DIR_NAME: &str = "notes";

fn ensure_notes_dir(roaming_dir: &Path) -> Result<PathBuf, String> {
    let dir = roaming_dir.join(NOTES_DIR_NAME);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create notes directory at {:?}: {e}", dir))?;
    Ok(dir)
}

fn note_file_path(notes_dir: &Path, slug: &str) -> Result<PathBuf, String> {
    validate_safe_slug(slug, "note")?;
    Ok(notes_dir.join(format!("{slug}.md")))
}

fn extract_heading_title(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("# ") {
            let trimmed = rest.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_note_title(content: &str, slug: &str) -> String {
    extract_heading_title(content).unwrap_or_else(|| slug.to_string())
}

fn unique_note_slug(notes_dir: &Path, title: &str, current_slug: Option<&str>) -> String {
    let base = {
        let s = slugify(title);
        if s.is_empty() {
            "untitled".to_string()
        } else {
            s
        }
    };

    if current_slug == Some(base.as_str()) || !notes_dir.join(format!("{base}.md")).exists() {
        return base;
    }

    let mut counter = 2usize;
    loop {
        let candidate = format!("{base}-{counter}");
        if current_slug == Some(candidate.as_str())
            || !notes_dir.join(format!("{candidate}.md")).exists()
        {
            return candidate;
        }
        counter += 1;
    }
}

fn replace_or_prepend_heading(content: &str, new_title: &str) -> String {
    let heading_line = format!("# {new_title}");
    let normalized = content.replace("\r\n", "\n");
    let mut lines: Vec<String> = normalized.lines().map(ToString::to_string).collect();
    let mut replaced = false;

    for line in &mut lines {
        if line.trim_start().strip_prefix("# ").is_some() {
            *line = heading_line.clone();
            replaced = true;
            break;
        }
    }

    if replaced {
        let mut out = lines.join("\n");
        if normalized.ends_with('\n') {
            out.push('\n');
        }
        out
    } else if normalized.trim().is_empty() {
        format!("{heading_line}\n")
    } else {
        format!("{heading_line}\n\n{normalized}")
    }
}

pub fn list_notes_in_dir(roaming_dir: &Path) -> Result<Vec<NoteSummary>, String> {
    let dir = ensure_notes_dir(roaming_dir)?;
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read notes directory at {:?}: {e}", dir))?;

    let mut notes = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read note entry in {:?}: {e}", dir))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_md = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if !is_md {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if validate_safe_slug(stem, "note").is_err() {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid UTF-8 filename at {:?}", path))?
            .to_string();
        let metadata = fs::metadata(&path)
            .map_err(|e| format!("Failed to read metadata for note {:?}: {e}", path))?;
        let updated_at_ms = file_modified_ms(&metadata)?;
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read note {:?}: {e}", path))?;

        notes.push(NoteSummary {
            slug: stem.to_string(),
            title: extract_note_title(&content, stem),
            filename,
            updated_at_ms,
        });
    }

    notes.sort_by(|a, b| {
        b.updated_at_ms
            .cmp(&a.updated_at_ms)
            .then_with(|| a.title.cmp(&b.title))
    });

    Ok(notes)
}

pub fn read_note_in_dir(roaming_dir: &Path, slug: &str) -> Result<NoteDocument, String> {
    let dir = ensure_notes_dir(roaming_dir)?;
    let path = note_file_path(&dir, slug)?;
    if !path.is_file() {
        return Err(format!("Note '{slug}' not found"));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read note '{slug}' at {:?}: {e}", path))?;
    let metadata = fs::metadata(&path)
        .map_err(|e| format!("Failed to read metadata for note '{slug}': {e}"))?;
    let updated_at_ms = file_modified_ms(&metadata)?;

    Ok(NoteDocument {
        slug: slug.to_string(),
        title: extract_note_title(&content, slug),
        filename: format!("{slug}.md"),
        content,
        updated_at_ms,
    })
}

pub fn save_note_in_dir(
    roaming_dir: &Path,
    slug: Option<String>,
    title: String,
    content: String,
) -> Result<NoteDocument, String> {
    let dir = ensure_notes_dir(roaming_dir)?;

    let (previous_slug, target_slug, final_content) = match slug {
        Some(existing_slug) => {
            validate_safe_slug(&existing_slug, "note")?;
            let old_path = note_file_path(&dir, &existing_slug)?;
            let old_title = if old_path.is_file() {
                let existing_content = fs::read_to_string(&old_path)
                    .map_err(|e| format!("Failed to read existing note '{existing_slug}': {e}"))?;
                Some(extract_note_title(&existing_content, &existing_slug))
            } else {
                None
            };

            let trimmed_title = title.trim();
            let heading_title = extract_heading_title(&content);
            let title_changed = !trimmed_title.is_empty()
                && old_title.as_deref() != Some(trimmed_title);

            let (effective_title, final_content) = if title_changed {
                (
                    trimmed_title.to_string(),
                    replace_or_prepend_heading(&content, trimmed_title),
                )
            } else if let Some(heading) = heading_title {
                (heading, content)
            } else if !trimmed_title.is_empty() {
                (trimmed_title.to_string(), content)
            } else {
                (existing_slug.clone(), content)
            };

            let new_slug = unique_note_slug(&dir, &effective_title, Some(&existing_slug));
            (Some(existing_slug), new_slug, final_content)
        }
        None => {
            let trimmed_title = title.trim();
            let effective_title = if !trimmed_title.is_empty() {
                trimmed_title.to_string()
            } else if let Some(heading) = extract_heading_title(&content) {
                heading
            } else {
                return Err("Note title cannot be empty".to_string());
            };

            let new_slug = unique_note_slug(&dir, &effective_title, None);
            let final_content = if extract_heading_title(&content).is_some() {
                content
            } else if content.trim().is_empty() {
                format!("# {effective_title}\n")
            } else {
                format!("# {effective_title}\n\n{content}")
            };
            (None, new_slug, final_content)
        }
    };

    let path = note_file_path(&dir, &target_slug)?;
    fs::write(&path, &final_content)
        .map_err(|e| format!("Failed to write note '{target_slug}' to {:?}: {e}", path))?;

    if let Some(old_slug) = previous_slug
        && old_slug != target_slug
    {
        let old_path = note_file_path(&dir, &old_slug)?;
        if old_path.is_file() {
            fs::remove_file(&old_path)
                .map_err(|e| format!("Failed to remove old note file '{old_slug}': {e}"))?;
        }
    }

    read_note_in_dir(roaming_dir, &target_slug)
}

pub fn rename_note_in_dir(
    roaming_dir: &Path,
    slug: &str,
    new_title: String,
) -> Result<NoteDocument, String> {
    let dir = ensure_notes_dir(roaming_dir)?;
    let old_path = note_file_path(&dir, slug)?;
    if !old_path.is_file() {
        return Err(format!("Note '{slug}' not found"));
    }

    let trimmed_title = new_title.trim();
    if trimmed_title.is_empty() {
        return Err("New note title cannot be empty".to_string());
    }

    let existing_content = fs::read_to_string(&old_path)
        .map_err(|e| format!("Failed to read note '{slug}' for rename: {e}"))?;
    let updated_content = replace_or_prepend_heading(&existing_content, trimmed_title);
    let new_slug = unique_note_slug(&dir, trimmed_title, Some(slug));
    let new_path = note_file_path(&dir, &new_slug)?;

    fs::write(&new_path, &updated_content)
        .map_err(|e| format!("Failed to write renamed note '{new_slug}': {e}"))?;

    if new_slug != slug {
        fs::remove_file(&old_path)
            .map_err(|e| format!("Failed to remove old note file '{slug}': {e}"))?;
    }

    read_note_in_dir(roaming_dir, &new_slug)
}

pub fn delete_note_in_dir(roaming_dir: &Path, slug: &str) -> Result<(), String> {
    let dir = ensure_notes_dir(roaming_dir)?;
    let path = note_file_path(&dir, slug)?;
    if !path.is_file() {
        return Err(format!("Note '{slug}' not found"));
    }
    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete note '{slug}' at {:?}: {e}", path))
}

fn resolve_notes_root(app: &AppHandle) -> Result<PathBuf, String> {
    super::resolve_user_workspace_root(app, LEGACY_NOTES_DIR_NAME, NOTES_DIR_NAME)
}

#[tauri::command]
pub fn workspace_list_notes(app: AppHandle) -> Result<Vec<NoteSummary>, String> {
    let root = resolve_notes_root(&app)?;
    list_notes_in_dir(&root)
}

#[tauri::command]
pub fn workspace_read_note(app: AppHandle, slug: String) -> Result<NoteDocument, String> {
    let root = resolve_notes_root(&app)?;
    read_note_in_dir(&root, &slug)
}

#[tauri::command]
pub fn workspace_save_note(
    app: AppHandle,
    slug: Option<String>,
    title: String,
    content: String,
) -> Result<NoteDocument, String> {
    let root = resolve_notes_root(&app)?;
    save_note_in_dir(&root, slug, title, content)
}

#[tauri::command]
pub fn workspace_rename_note(
    app: AppHandle,
    slug: String,
    new_title: String,
) -> Result<NoteDocument, String> {
    let root = resolve_notes_root(&app)?;
    rename_note_in_dir(&root, &slug, new_title)
}

#[tauri::command]
pub fn workspace_delete_note(app: AppHandle, slug: String) -> Result<(), String> {
    let root = resolve_notes_root(&app)?;
    delete_note_in_dir(&root, &slug)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::temp_roaming_dir;

    #[test]
    fn test_notes_crud_heading_extraction_and_rename() {
        let dir = temp_roaming_dir("caffeine_notes_test");

        let created = save_note_in_dir(
            &dir,
            None,
            "Network Runbook".to_string(),
            "Verify VLAN trunking.".to_string(),
        )
        .unwrap();
        assert_eq!(created.slug, "network-runbook");
        assert_eq!(created.filename, "network-runbook.md");
        assert_eq!(created.title, "Network Runbook");
        assert!(created.content.starts_with("# Network Runbook\n\n"));

        let duplicate = save_note_in_dir(
            &dir,
            None,
            "Network Runbook".to_string(),
            "# Custom Header\nBody".to_string(),
        )
        .unwrap();
        assert_eq!(duplicate.slug, "network-runbook-2");
        assert_eq!(duplicate.title, "Custom Header");

        let list = list_notes_in_dir(&dir).unwrap();
        assert_eq!(list.len(), 2);

        let renamed =
            rename_note_in_dir(&dir, &created.slug, "Core Switch Runbook".to_string()).unwrap();
        assert_eq!(renamed.slug, "core-switch-runbook");
        assert_eq!(renamed.title, "Core Switch Runbook");
        assert!(!dir.join("notes/network-runbook.md").exists());

        let read_back = read_note_in_dir(&dir, "core-switch-runbook").unwrap();
        assert_eq!(read_back.title, "Core Switch Runbook");

        let renamed_on_title = save_note_in_dir(
            &dir,
            Some("core-switch-runbook".to_string()),
            "Edge Switch Runbook".to_string(),
            "# Core Switch Runbook\nUpdated body".to_string(),
        )
        .unwrap();
        assert_eq!(renamed_on_title.slug, "edge-switch-runbook");
        assert_eq!(renamed_on_title.filename, "edge-switch-runbook.md");
        assert_eq!(renamed_on_title.title, "Edge Switch Runbook");
        assert!(!dir.join("Notes/core-switch-runbook.md").exists());

        let renamed_on_heading = save_note_in_dir(
            &dir,
            Some("edge-switch-runbook".to_string()),
            "Edge Switch Runbook".to_string(),
            "# Core Switch Runbook v2\nUpdated body".to_string(),
        )
        .unwrap();
        assert_eq!(renamed_on_heading.slug, "core-switch-runbook-v2");
        assert_eq!(renamed_on_heading.filename, "core-switch-runbook-v2.md");
        assert_eq!(renamed_on_heading.title, "Core Switch Runbook v2");
        assert!(!dir.join("Notes/edge-switch-runbook.md").exists());

        delete_note_in_dir(&dir, "core-switch-runbook-v2").unwrap();
        assert!(read_note_in_dir(&dir, "core-switch-runbook-v2").is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_notes_rejects_path_traversal_slugs() {
        let dir = temp_roaming_dir("caffeine_notes_traversal");

        for bad_slug in ["../secret", "..\\secret", "a/b", ".", "..", "", ".hidden"] {
            assert!(
                read_note_in_dir(&dir, bad_slug).is_err(),
                "expected read_note_in_dir to reject '{bad_slug}'"
            );
            assert!(
                save_note_in_dir(&dir, Some(bad_slug.to_string()), "x".into(), "y".into()).is_err(),
                "expected save_note_in_dir to reject '{bad_slug}'"
            );
            assert!(
                rename_note_in_dir(&dir, bad_slug, "Valid".into()).is_err(),
                "expected rename_note_in_dir to reject '{bad_slug}'"
            );
            assert!(
                delete_note_in_dir(&dir, bad_slug).is_err(),
                "expected delete_note_in_dir to reject '{bad_slug}'"
            );
        }

        let _ = fs::remove_dir_all(&dir);
    }
}
