use super::types::{DetectedScriptLanguage, ScriptDocument, ScriptSummary};
use super::{file_modified_ms, slugify, validate_safe_slug};
use crate::process::find_executable_in_path;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

const SCRIPTS_DIR_NAME: &str = "scripts";
const SCRIPTS_META_FILE: &str = ".scripts-meta.json";
const INDENT_UNIT: &str = "    ";

const LANGUAGE_SPECS: &[(&str, &str)] = &[
    ("powershell", "ps1"),
    ("cmd", "cmd"),
    ("bat", "bat"),
    ("sh", "sh"),
    ("bash", "bash"),
    ("zsh", "zsh"),
    ("python", "py"),
];

fn extension_for_language(language_id: &str) -> Result<&'static str, String> {
    let normalized = language_id.trim().to_ascii_lowercase();
    LANGUAGE_SPECS
        .iter()
        .find(|(id, _)| *id == normalized)
        .map(|(_, ext)| *ext)
        .ok_or_else(|| format!("Unsupported script language '{language_id}'"))
}

fn language_for_extension(ext: &str) -> Option<&'static str> {
    let normalized = ext.trim().to_ascii_lowercase();
    LANGUAGE_SPECS
        .iter()
        .find(|(_, e)| *e == normalized)
        .map(|(id, _)| *id)
}

fn ensure_scripts_dir(roaming_dir: &Path) -> Result<PathBuf, String> {
    let dir = roaming_dir.join(SCRIPTS_DIR_NAME);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create scripts directory at {:?}: {e}", dir))?;
    Ok(dir)
}

fn load_scripts_meta(scripts_dir: &Path) -> Result<HashMap<String, String>, String> {
    let meta_path = scripts_dir.join(SCRIPTS_META_FILE);
    if !meta_path.exists() {
        return Ok(HashMap::new());
    }
    let raw = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read scripts metadata at {:?}: {e}", meta_path))?;
    serde_json::from_str::<HashMap<String, String>>(&raw)
        .map_err(|e| format!("Failed to parse scripts metadata JSON at {:?}: {e}", meta_path))
}

fn save_scripts_meta(scripts_dir: &Path, meta: &HashMap<String, String>) -> Result<(), String> {
    let meta_path = scripts_dir.join(SCRIPTS_META_FILE);
    let serialized = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("Failed to serialize scripts metadata: {e}"))?;
    fs::write(&meta_path, serialized)
        .map_err(|e| format!("Failed to write scripts metadata at {:?}: {e}", meta_path))
}

fn find_script_file(scripts_dir: &Path, id: &str) -> Result<Option<(PathBuf, String, String)>, String> {
    validate_safe_slug(id, "script")?;
    for (lang_id, ext) in LANGUAGE_SPECS {
        let candidate = scripts_dir.join(format!("{id}.{ext}"));
        if candidate.is_file() {
            return Ok(Some((
                candidate,
                (*ext).to_string(),
                (*lang_id).to_string(),
            )));
        }
    }
    Ok(None)
}

fn unique_script_id(scripts_dir: &Path, name: &str) -> Result<String, String> {
    let base = {
        let s = slugify(name);
        if s.is_empty() {
            "script".to_string()
        } else {
            s
        }
    };

    if find_script_file(scripts_dir, &base)?.is_none() {
        return Ok(base);
    }

    let mut counter = 2usize;
    loop {
        let candidate = format!("{base}-{counter}");
        if find_script_file(scripts_dir, &candidate)?.is_none() {
            return Ok(candidate);
        }
        counter += 1;
    }
}

pub fn detect_script_languages() -> Result<Vec<DetectedScriptLanguage>, String> {
    let mut languages = Vec::new();

    #[cfg(windows)]
    {
        if let Some(ps_path) =
            find_executable_in_path("pwsh").or_else(|| find_executable_in_path("powershell"))
        {
            languages.push(DetectedScriptLanguage {
                id: "powershell".to_string(),
                label: "PowerShell (.ps1)".to_string(),
                extension: "ps1".to_string(),
                executable: ps_path.to_string_lossy().to_string(),
            });
        }
        if let Some(cmd_path) = find_executable_in_path("cmd") {
            let exe = cmd_path.to_string_lossy().to_string();
            languages.push(DetectedScriptLanguage {
                id: "cmd".to_string(),
                label: "Command Script (.cmd)".to_string(),
                extension: "cmd".to_string(),
                executable: exe.clone(),
            });
            languages.push(DetectedScriptLanguage {
                id: "bat".to_string(),
                label: "Batch File (.bat)".to_string(),
                extension: "bat".to_string(),
                executable: exe,
            });
        }
        if let Some(sh_path) =
            find_executable_in_path("bash").or_else(|| find_executable_in_path("sh"))
        {
            languages.push(DetectedScriptLanguage {
                id: "sh".to_string(),
                label: "Shell Script (.sh)".to_string(),
                extension: "sh".to_string(),
                executable: sh_path.to_string_lossy().to_string(),
            });
        }
        if let Some(py_path) =
            find_executable_in_path("python").or_else(|| find_executable_in_path("python3"))
        {
            languages.push(DetectedScriptLanguage {
                id: "python".to_string(),
                label: "Python (.py)".to_string(),
                extension: "py".to_string(),
                executable: py_path.to_string_lossy().to_string(),
            });
        }
    }

    #[cfg(not(windows))]
    {
        let resolve_unix = |names: &[&str], fallbacks: &[&str]| -> Option<String> {
            for name in names {
                if let Some(found) = find_executable_in_path(name) {
                    return Some(found.to_string_lossy().to_string());
                }
            }
            for fallback in fallbacks {
                if Path::new(fallback).is_file() {
                    return Some((*fallback).to_string());
                }
            }
            None
        };

        if let Some(exe) = resolve_unix(&["sh"], &["/bin/sh", "/usr/bin/sh"]) {
            languages.push(DetectedScriptLanguage {
                id: "sh".to_string(),
                label: "Bourne Shell (.sh)".to_string(),
                extension: "sh".to_string(),
                executable: exe,
            });
        }
        if let Some(exe) = resolve_unix(&["bash"], &["/bin/bash", "/usr/bin/bash"]) {
            languages.push(DetectedScriptLanguage {
                id: "bash".to_string(),
                label: "Bash (.bash)".to_string(),
                extension: "bash".to_string(),
                executable: exe,
            });
        }
        if let Some(exe) = resolve_unix(&["zsh"], &["/bin/zsh", "/usr/bin/zsh"]) {
            languages.push(DetectedScriptLanguage {
                id: "zsh".to_string(),
                label: "Zsh (.zsh)".to_string(),
                extension: "zsh".to_string(),
                executable: exe,
            });
        }
        if let Some(exe) = resolve_unix(&["pwsh"], &["/usr/bin/pwsh", "/usr/local/bin/pwsh"]) {
            languages.push(DetectedScriptLanguage {
                id: "powershell".to_string(),
                label: "PowerShell (.ps1)".to_string(),
                extension: "ps1".to_string(),
                executable: exe,
            });
        }
        if let Some(exe) = resolve_unix(
            &["python3", "python"],
            &["/usr/bin/python3", "/usr/local/bin/python3", "/usr/bin/python"],
        ) {
            languages.push(DetectedScriptLanguage {
                id: "python".to_string(),
                label: "Python (.py)".to_string(),
                extension: "py".to_string(),
                executable: exe,
            });
        }
    }

    Ok(languages)
}

pub fn list_scripts_in_dir(roaming_dir: &Path) -> Result<Vec<ScriptSummary>, String> {
    let dir = ensure_scripts_dir(roaming_dir)?;
    let meta = load_scripts_meta(&dir)?;
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read scripts directory at {:?}: {e}", dir))?;

    let mut scripts = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read script entry in {:?}: {e}", dir))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let Some(language_id) = language_for_extension(ext) else {
            continue;
        };
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if validate_safe_slug(stem, "script").is_err() {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid UTF-8 filename at {:?}", path))?
            .to_string();
        let metadata = fs::metadata(&path)
            .map_err(|e| format!("Failed to read metadata for script {:?}: {e}", path))?;
        let updated_at_ms = file_modified_ms(&metadata)?;
        let name = meta
            .get(stem)
            .cloned()
            .unwrap_or_else(|| stem.to_string());

        scripts.push(ScriptSummary {
            id: stem.to_string(),
            name,
            filename,
            extension: ext.to_ascii_lowercase(),
            language_id: language_id.to_string(),
            updated_at_ms,
        });
    }

    scripts.sort_by(|a, b| {
        b.updated_at_ms
            .cmp(&a.updated_at_ms)
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(scripts)
}

pub fn read_script_in_dir(roaming_dir: &Path, id: &str) -> Result<ScriptDocument, String> {
    let dir = ensure_scripts_dir(roaming_dir)?;
    let (path, extension, language_id) =
        find_script_file(&dir, id)?.ok_or_else(|| format!("Script '{id}' not found"))?;
    let meta = load_scripts_meta(&dir)?;
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read script '{id}' at {:?}: {e}", path))?;
    let metadata = fs::metadata(&path)
        .map_err(|e| format!("Failed to read metadata for script '{id}': {e}"))?;
    let updated_at_ms = file_modified_ms(&metadata)?;
    let filename = format!("{id}.{extension}");
    let name = meta.get(id).cloned().unwrap_or_else(|| id.to_string());

    Ok(ScriptDocument {
        id: id.to_string(),
        name,
        filename,
        extension,
        language_id,
        content,
        path: path.to_string_lossy().to_string(),
        updated_at_ms,
    })
}

pub fn save_script_in_dir(
    roaming_dir: &Path,
    id: Option<String>,
    name: String,
    language_id: String,
    content: String,
) -> Result<ScriptDocument, String> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err("Script name cannot be empty".to_string());
    }

    let normalized_lang = language_id.trim().to_ascii_lowercase();
    let ext = extension_for_language(&normalized_lang)?;
    let dir = ensure_scripts_dir(roaming_dir)?;

    let target_id = match id {
        Some(existing_id) => {
            validate_safe_slug(&existing_id, "script")?;
            if let Some((old_path, old_ext, _)) = find_script_file(&dir, &existing_id)?
                && old_ext != ext
            {
                fs::remove_file(&old_path).map_err(|e| {
                    format!(
                        "Failed to remove previous script file {:?} on language change: {e}",
                        old_path
                    )
                })?;
            }
            existing_id
        }
        None => unique_script_id(&dir, trimmed_name)?,
    };

    let target_path = dir.join(format!("{target_id}.{ext}"));
    fs::write(&target_path, &content)
        .map_err(|e| format!("Failed to write script '{target_id}' to {:?}: {e}", target_path))?;

    let mut meta = load_scripts_meta(&dir)?;
    meta.insert(target_id.clone(), trimmed_name.to_string());
    save_scripts_meta(&dir, &meta)?;

    read_script_in_dir(roaming_dir, &target_id)
}

pub fn delete_script_in_dir(roaming_dir: &Path, id: &str) -> Result<(), String> {
    let dir = ensure_scripts_dir(roaming_dir)?;
    let (path, _, _) =
        find_script_file(&dir, id)?.ok_or_else(|| format!("Script '{id}' not found"))?;
    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete script '{id}' at {:?}: {e}", path))?;

    let mut meta = load_scripts_meta(&dir)?;
    if meta.remove(id).is_some() {
        save_scripts_meta(&dir, &meta)?;
    }

    Ok(())
}

pub fn build_script_run_command_in_dir(roaming_dir: &Path, id: &str) -> Result<String, String> {
    let doc = read_script_in_dir(roaming_dir, id)?;

    #[cfg(windows)]
    {
        let escaped = doc.path.replace('"', "`\"");
        let cmd = match doc.language_id.as_str() {
            "powershell" => {
                let ps_exe = if find_executable_in_path("pwsh").is_some() {
                    "pwsh"
                } else {
                    "powershell"
                };
                format!("{ps_exe} -NoProfile -ExecutionPolicy Bypass -File \"{escaped}\"")
            }
            "cmd" | "bat" => format!("cmd /c \"{escaped}\""),
            "sh" | "bash" => format!("bash \"{escaped}\""),
            "zsh" => format!("zsh \"{escaped}\""),
            "python" => format!("python \"{escaped}\""),
            other => return Err(format!("Unsupported script language '{other}'")),
        };
        Ok(format!("{cmd}\r"))
    }

    #[cfg(not(windows))]
    {
        let escaped = doc.path.replace('\'', "'\\''");
        let cmd = match doc.language_id.as_str() {
            "sh" => format!("sh '{escaped}'"),
            "bash" => format!("bash '{escaped}'"),
            "zsh" => format!("zsh '{escaped}'"),
            "powershell" => format!("pwsh -NoProfile -File '{escaped}'"),
            "python" => {
                let py_exe = if find_executable_in_path("python3").is_some() {
                    "python3"
                } else {
                    "python"
                };
                format!("{py_exe} '{escaped}'")
            }
            "cmd" | "bat" => {
                return Err(format!(
                    "Script language '{}' is only supported on Windows",
                    doc.language_id
                ))
            }
            other => return Err(format!("Unsupported script language '{other}'")),
        };
        Ok(format!("{cmd}\r"))
    }
}

fn strip_hash_comment_outside_quotes(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for ch in line.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && !in_single {
            out.push(ch);
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            out.push(ch);
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            out.push(ch);
            continue;
        }
        if ch == '#' && !in_single && !in_double {
            break;
        }
        if !in_single && !in_double {
            out.push(ch);
        }
    }
    out
}

fn count_delimiters(code: &str, open: char, close: char) -> (usize, usize, usize) {
    let trimmed = code.trim();
    let mut leading_close = 0usize;
    for ch in trimmed.chars() {
        if ch == close {
            leading_close += 1;
        } else if !ch.is_whitespace() {
            break;
        }
    }

    let mut opens = 0usize;
    let mut closes = 0usize;
    for ch in trimmed.chars() {
        if ch == open {
            opens += 1;
        } else if ch == close {
            closes += 1;
        }
    }
    (leading_close, opens, closes)
}

fn tokenize_shell_words(code: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in code.chars() {
        if ch == '{' || ch == '}' || ch == ';' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            if ch == '{' || ch == '}' {
                tokens.push(ch.to_string());
            }
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn shell_indent_step(line: &str) -> (usize, isize) {
    let code = strip_hash_comment_outside_quotes(line);
    let tokens = tokenize_shell_words(&code);
    if tokens.is_empty() {
        return (0, 0);
    }

    let (leading_dedent, start_idx, base_after) = if tokens[0] == "else" {
        (1usize, 1usize, 1isize)
    } else if tokens[0] == "elif" {
        (1usize, 1usize, 0isize)
    } else {
        let mut dedent = 0usize;
        for tok in &tokens {
            if matches!(tok.as_str(), "}" | "fi" | "done" | "esac") {
                dedent += 1;
            } else {
                break;
            }
        }
        (dedent, dedent, 0isize)
    };

    let mut after_delta = base_after;
    for tok in &tokens[start_idx..] {
        match tok.as_str() {
            "{" | "then" | "do" | "case" => after_delta += 1,
            "}" | "fi" | "done" | "esac" => after_delta -= 1,
            _ => {}
        }
    }

    (leading_dedent, after_delta)
}

fn is_batch_comment(trimmed: &str) -> bool {
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("rem ") || lower == "rem" || trimmed.starts_with("::")
}

fn strip_double_quoted_for_batch(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_double = false;
    for ch in line.chars() {
        if ch == '"' {
            in_double = !in_double;
        } else if !in_double {
            out.push(ch);
        }
    }
    out
}

fn apply_block_indentation<F>(lines: &[String], mut step_fn: F) -> Vec<String>
where
    F: FnMut(&str) -> (usize, isize),
{
    let mut out = Vec::with_capacity(lines.len());
    let mut indent: usize = 0;

    for raw_line in lines {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            out.push(String::new());
            continue;
        }

        let (leading_dedent, after_delta) = step_fn(trimmed);
        indent = indent.saturating_sub(leading_dedent);
        out.push(format!("{}{}", INDENT_UNIT.repeat(indent), trimmed));

        if after_delta >= 0 {
            indent = indent.saturating_add(after_delta as usize);
        } else {
            indent = indent.saturating_sub(after_delta.unsigned_abs());
        }
    }

    out
}

pub fn format_script_source(language_id: &str, content: &str) -> Result<String, String> {
    let normalized_lang = language_id.trim().to_ascii_lowercase();
    extension_for_language(&normalized_lang)?;

    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = normalized
        .lines()
        .map(|l| l.trim_end().to_string())
        .collect();

    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        return Ok(String::new());
    }

    let formatted_lines = match normalized_lang.as_str() {
        "powershell" => apply_block_indentation(&lines, |line| {
            let code = strip_hash_comment_outside_quotes(line);
            let (leading, opens, closes) = count_delimiters(&code, '{', '}');
            let net = (opens as isize) - (closes as isize) + (leading as isize);
            (leading, net)
        }),
        "sh" | "bash" | "zsh" => apply_block_indentation(&lines, shell_indent_step),
        "cmd" | "bat" => apply_block_indentation(&lines, |line| {
            if is_batch_comment(line) {
                return (0, 0);
            }
            let code = strip_double_quoted_for_batch(line);
            let (leading, opens, closes) = count_delimiters(&code, '(', ')');
            let net = (opens as isize) - (closes as isize) + (leading as isize);
            (leading, net)
        }),
        // Python indentation is semantically significant; preserve leading whitespace.
        "python" => lines,
        other => return Err(format!("Unsupported script language '{other}'")),
    };

    Ok(format!("{}\n", formatted_lines.join("\n")))
}

#[tauri::command]
pub fn workspace_detect_script_languages() -> Result<Vec<DetectedScriptLanguage>, String> {
    detect_script_languages()
}

#[tauri::command]
pub fn workspace_list_scripts(app: AppHandle) -> Result<Vec<ScriptSummary>, String> {
    let roaming_dir = crate::config::get_user_roaming_dir(&app)?;
    list_scripts_in_dir(&roaming_dir)
}

#[tauri::command]
pub fn workspace_read_script(app: AppHandle, id: String) -> Result<ScriptDocument, String> {
    let roaming_dir = crate::config::get_user_roaming_dir(&app)?;
    read_script_in_dir(&roaming_dir, &id)
}

#[tauri::command]
pub fn workspace_save_script(
    app: AppHandle,
    id: Option<String>,
    name: String,
    language_id: String,
    content: String,
) -> Result<ScriptDocument, String> {
    let roaming_dir = crate::config::get_user_roaming_dir(&app)?;
    save_script_in_dir(&roaming_dir, id, name, language_id, content)
}

#[tauri::command]
pub fn workspace_delete_script(app: AppHandle, id: String) -> Result<(), String> {
    let roaming_dir = crate::config::get_user_roaming_dir(&app)?;
    delete_script_in_dir(&roaming_dir, &id)
}

#[tauri::command]
pub fn workspace_format_script(language_id: String, content: String) -> Result<String, String> {
    format_script_source(&language_id, &content)
}

#[tauri::command]
pub fn workspace_build_script_run_command(app: AppHandle, id: String) -> Result<String, String> {
    let roaming_dir = crate::config::get_user_roaming_dir(&app)?;
    build_script_run_command_in_dir(&roaming_dir, &id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_roaming_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{}_{nanos}", std::process::id()))
    }

    #[test]
    fn test_detect_script_languages_returns_native_defaults() {
        let langs = detect_script_languages().unwrap();
        assert!(!langs.is_empty());

        #[cfg(windows)]
        {
            assert!(langs.iter().any(|l| l.id == "powershell" && l.extension == "ps1"));
            assert!(langs.iter().any(|l| l.id == "cmd" && l.extension == "cmd"));
            assert!(langs.iter().any(|l| l.id == "bat" && l.extension == "bat"));
        }

        #[cfg(not(windows))]
        {
            assert!(langs.iter().any(|l| l.id == "sh" && l.extension == "sh"));
        }
    }

    #[test]
    fn test_scripts_crud_language_switch_and_run_command() {
        let dir = temp_roaming_dir("caffeine_scripts_test");

        let saved = save_script_in_dir(
            &dir,
            None,
            "Flush DNS".to_string(),
            "powershell".to_string(),
            "Write-Host 'Flushing'\n".to_string(),
        )
        .unwrap();
        assert_eq!(saved.id, "flush-dns");
        assert_eq!(saved.name, "Flush DNS");
        assert_eq!(saved.extension, "ps1");
        assert_eq!(saved.filename, "flush-dns.ps1");

        let list = list_scripts_in_dir(&dir).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Flush DNS");

        #[cfg(windows)]
        let expected_lang = "bat";
        #[cfg(not(windows))]
        let expected_lang = "sh";

        let updated = save_script_in_dir(
            &dir,
            Some(saved.id.clone()),
            "Flush DNS Batch".to_string(),
            expected_lang.to_string(),
            "echo done\n".to_string(),
        )
        .unwrap();
        assert_eq!(updated.id, "flush-dns");
        assert_eq!(updated.name, "Flush DNS Batch");
        assert!(!dir.join("scripts/flush-dns.ps1").exists());

        let run_cmd = build_script_run_command_in_dir(&dir, &updated.id).unwrap();
        assert!(run_cmd.ends_with('\r'));
        assert!(run_cmd.contains("flush-dns"));

        delete_script_in_dir(&dir, &updated.id).unwrap();
        assert!(read_script_in_dir(&dir, &updated.id).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scripts_rejects_traversal_and_unknown_language() {
        let dir = temp_roaming_dir("caffeine_scripts_invalid");

        assert!(
            save_script_in_dir(&dir, None, "Bad".into(), "ruby".into(), "puts 1".into()).is_err()
        );
        assert!(read_script_in_dir(&dir, "../evil").is_err());
        assert!(delete_script_in_dir(&dir, "..\\evil").is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_format_script_powershell_shell_and_batch() {
        let ps = "if ($true) {\r\nWrite-Host \"a\"   \r\n} else {\r\nWrite-Host \"b\"\r\n}\r\n\r\n";
        let formatted_ps = format_script_source("powershell", ps).unwrap();
        assert_eq!(
            formatted_ps,
            "if ($true) {\n    Write-Host \"a\"\n} else {\n    Write-Host \"b\"\n}\n"
        );

        let sh = "if [ -f foo ]; then\necho yes   \nelif [ -f bar ]; then\necho maybe\nelse\necho no\nfi\nfor i in 1 2; do\necho $i\ndone\n";
        let formatted_sh = format_script_source("sh", sh).unwrap();
        assert_eq!(
            formatted_sh,
            "if [ -f foo ]; then\n    echo yes\nelif [ -f bar ]; then\n    echo maybe\nelse\n    echo no\nfi\nfor i in 1 2; do\n    echo $i\ndone\n"
        );

        let bat = "if exist test.txt (\necho found  \n) else (\necho missing\n)\n";
        let formatted_bat = format_script_source("bat", bat).unwrap();
        assert_eq!(
            formatted_bat,
            "if exist test.txt (\n    echo found\n) else (\n    echo missing\n)\n"
        );

        let py = "def hello():\r\n    print('hi')   \r\n\r\n";
        let formatted_py = format_script_source("python", py).unwrap();
        assert_eq!(formatted_py, "def hello():\n    print('hi')\n");

        let quoted_ps = "if ($true) {\nWrite-Host \"}\" # { ignored }\n}\n";
        assert_eq!(
            format_script_source("powershell", quoted_ps).unwrap(),
            "if ($true) {\n    Write-Host \"}\" # { ignored }\n}\n"
        );

        let multiline_sh = "if cond1\nthen\necho 1\nelif cond2\nthen\necho 2\nfi\n";
        assert_eq!(
            format_script_source("bash", multiline_sh).unwrap(),
            "if cond1\nthen\n    echo 1\nelif cond2\nthen\n    echo 2\nfi\n"
        );
    }
}
