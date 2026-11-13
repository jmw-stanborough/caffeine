pub mod db;
pub mod notes;
pub mod script_formatter;
pub mod scripts;
pub mod todo;
pub mod types;
pub mod worker;

use std::fs::Metadata;
use std::time::{SystemTime, UNIX_EPOCH};

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
