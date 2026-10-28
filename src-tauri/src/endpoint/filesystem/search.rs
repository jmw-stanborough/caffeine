use super::types::{
    FileEntry, FilterCondition, FilterCriterion, FilterProperty, FilterTarget, OperationProgress,
};
use regex::Regex;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const MAX_SEARCH_CONTENT_BYTES: u64 = 20 * 1024 * 1024;
const PROGRESS_THROTTLE_INTERVAL: Duration = Duration::from_millis(120);

pub fn matches_criteria(
    entry: &FileEntry,
    criteria: &[FilterCriterion],
    match_all: bool,
) -> bool {
    if criteria.is_empty() {
        return true;
    }

    if match_all {
        criteria.iter().all(|c| evaluate_criterion(entry, c))
    } else {
        criteria.iter().any(|c| evaluate_criterion(entry, c))
    }
}

pub fn evaluate_criterion(entry: &FileEntry, criterion: &FilterCriterion) -> bool {
    match criterion.target {
        FilterTarget::Files if entry.is_dir => return false,
        FilterTarget::Directories if !entry.is_dir => return false,
        _ => {}
    }

    match criterion.property {
        FilterProperty::Name => match_text_condition(&entry.name, &criterion.condition, &criterion.value),
        FilterProperty::Path => match_text_condition(&entry.path, &criterion.condition, &criterion.value),
        FilterProperty::Extension => {
            let ext = entry.extension.as_deref().unwrap_or("");
            let clean_value = criterion.value
                .trim_start_matches("*.")
                .trim_start_matches('.');
            match_text_condition(ext, &criterion.condition, clean_value)
        }
        FilterProperty::Size => {
            let target_size = criterion.value.parse::<u64>().unwrap_or(0);
            match criterion.condition {
                FilterCondition::Gt => entry.size_bytes > target_size,
                FilterCondition::Lt => entry.size_bytes < target_size,
                FilterCondition::Is => entry.size_bytes == target_size,
                _ => false,
            }
        }
        FilterProperty::Content => {
            if entry.is_dir || entry.size_bytes > MAX_SEARCH_CONTENT_BYTES {
                return false;
            }
            search_file_content(Path::new(&entry.path), &criterion.condition, &criterion.value)
        }
    }
}

fn match_text_condition(actual: &str, condition: &FilterCondition, expected: &str) -> bool {
    let actual_lower = actual.to_lowercase();
    let expected_lower = expected.to_lowercase();

    if expected_lower.contains('*') || expected_lower.contains('?') {
        let pattern = match condition {
            FilterCondition::Is => glob_to_regex(&expected_lower),
            FilterCondition::StartsWith => {
                let trimmed = expected_lower.trim_end_matches('*');
                format!("^{}.*$", glob_body(trimmed))
            }
            FilterCondition::EndsWith => {
                let trimmed = expected_lower.trim_start_matches('*');
                format!("^.*{}$", glob_body(trimmed))
            }
            FilterCondition::Contains | FilterCondition::DoesNotContain => {
                let trimmed = expected_lower.trim_matches('*');
                format!("^.*{}.*$", glob_body(trimmed))
            }
            FilterCondition::Matches => expected.to_string(),
            _ => return false,
        };

        if let Ok(re) = Regex::new(&pattern) {
            let matched = re.is_match(&actual_lower);
            return if *condition == FilterCondition::DoesNotContain {
                !matched
            } else {
                matched
            };
        }
    }

    match condition {
        FilterCondition::Is => actual_lower == expected_lower,
        FilterCondition::StartsWith => actual_lower.starts_with(&expected_lower),
        FilterCondition::EndsWith => actual_lower.ends_with(&expected_lower),
        FilterCondition::Contains => actual_lower.contains(&expected_lower),
        FilterCondition::DoesNotContain => !actual_lower.contains(&expected_lower),
        FilterCondition::Matches => {
            Regex::new(expected)
                .map(|re| re.is_match(actual))
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn glob_body(pattern: &str) -> String {
    let mut regex = String::new();
    for c in pattern.chars() {
        match c {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '[' | ']' | '{' | '}' | '\\' => {
                regex.push('\\');
                regex.push(c);
            }
            _ => regex.push(c),
        }
    }
    regex
}

fn glob_to_regex(pattern: &str) -> String {
    format!("^{}$", glob_body(pattern))
}

fn search_file_content(path: &Path, condition: &FilterCondition, query: &str) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut reader = BufReader::with_capacity(32 * 1024, file);
    let mut line = String::new();
    let regex = if *condition == FilterCondition::Matches {
        Regex::new(query).ok()
    } else {
        None
    };

    let query_lower = query.to_lowercase();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if let Some(ref re) = regex {
                    if re.is_match(&line) {
                        return true;
                    }
                } else if line.to_lowercase().contains(&query_lower) {
                    return true;
                }
            }
            Err(_) => break,
        }
    }

    false
}

pub fn search_tree<F>(
    root_path: &Path,
    criteria: &[FilterCriterion],
    match_all: bool,
    drive_total_bytes: u64,
    task_id: &str,
    cancel: &AtomicBool,
    mut on_progress: F,
) -> Result<Vec<FileEntry>, String>
where
    F: FnMut(OperationProgress),
{
    if !root_path.exists() {
        return Err(format!("Root search path does not exist: {}", root_path.display()));
    }

    let mut matched_entries = Vec::new();
    let mut stack = vec![root_path.to_path_buf()];
    let mut items_processed = 0u64;
    let mut bytes_processed = 0u64;
    let mut last_progress = Instant::now();

    while let Some(current) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Search cancelled by user".to_string());
        }

        let read_dir = match fs::read_dir(&current) {
            Ok(rd) => rd,
            Err(_) => continue,
        };

        for item in read_dir {
            if cancel.load(Ordering::Relaxed) {
                return Err("Search cancelled by user".to_string());
            }

            let entry = match item {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let is_dir = meta.is_dir();
            let size_bytes = if is_dir { 0 } else { meta.len() };
            bytes_processed = bytes_processed.saturating_add(size_bytes);
            items_processed += 1;
            if items_processed % 1024 == 0 {
                std::thread::yield_now();
            }

            if is_dir && !meta.file_type().is_symlink() {
                stack.push(path.clone());
            }

            let drive_usage_percent = if drive_total_bytes > 0 {
                (size_bytes as f64 / drive_total_bytes as f64) * 100.0
            } else {
                0.0
            };

            let file_entry = FileEntry {
                path: path.to_string_lossy().to_string(),
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir,
                size_bytes,
                drive_usage_percent,
                modified_timestamp: meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64),
                extension: if is_dir {
                    None
                } else {
                    path.extension().map(|e| e.to_string_lossy().to_string())
                },
                child_count: None,
            };

            if matches_criteria(&file_entry, criteria, match_all) {
                matched_entries.push(file_entry);
            }

            if last_progress.elapsed() >= PROGRESS_THROTTLE_INTERVAL {
                last_progress = Instant::now();
                on_progress(OperationProgress {
                    task_id: task_id.to_string(),
                    operation: "search".to_string(),
                    items_processed,
                    total_estimated: None,
                    bytes_processed,
                    current_path: path.to_string_lossy().to_string(),
                    is_complete: false,
                    error: None,
                });
            }
        }
    }

    matched_entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    Ok(matched_entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_name_conditions() {
        let entry = FileEntry {
            path: "/var/log/syslog.1".to_string(),
            name: "syslog.1".to_string(),
            is_dir: false,
            size_bytes: 1024,
            drive_usage_percent: 0.1,
            modified_timestamp: None,
            extension: Some("1".to_string()),
            child_count: None,
        };

        let is_criterion = FilterCriterion {
            target: FilterTarget::Files,
            property: FilterProperty::Name,
            condition: FilterCondition::Is,
            value: "syslog.1".to_string(),
        };
        assert!(evaluate_criterion(&entry, &is_criterion));

        let ends_criterion = FilterCriterion {
            target: FilterTarget::Files,
            property: FilterProperty::Name,
            condition: FilterCondition::EndsWith,
            value: ".1".to_string(),
        };
        assert!(evaluate_criterion(&entry, &ends_criterion));

        let regex_criterion = FilterCriterion {
            target: FilterTarget::Files,
            property: FilterProperty::Name,
            condition: FilterCondition::Matches,
            value: r"^syslog\.\d+$".to_string(),
        };
        assert!(evaluate_criterion(&entry, &regex_criterion));
    }

    #[test]
    fn test_filter_size_and_target() {
        let dir_entry = FileEntry {
            path: "/tmp/cache".to_string(),
            name: "cache".to_string(),
            is_dir: true,
            size_bytes: 5000,
            drive_usage_percent: 0.5,
            modified_timestamp: None,
            extension: None,
            child_count: Some(10),
        };

        let file_only = FilterCriterion {
            target: FilterTarget::Files,
            property: FilterProperty::Name,
            condition: FilterCondition::Contains,
            value: "cache".to_string(),
        };
        assert!(!evaluate_criterion(&dir_entry, &file_only));

        let dir_criterion = FilterCriterion {
            target: FilterTarget::Directories,
            property: FilterProperty::Size,
            condition: FilterCondition::Gt,
            value: "4000".to_string(),
        };
        assert!(evaluate_criterion(&dir_entry, &dir_criterion));
    }

    #[test]
    fn test_filter_extension_wildcards() {
        let entry = FileEntry {
            path: "/data/report.BESPOKE-EXTENSION-ABC".to_string(),
            name: "report.BESPOKE-EXTENSION-ABC".to_string(),
            is_dir: false,
            size_bytes: 2048,
            drive_usage_percent: 0.1,
            modified_timestamp: None,
            extension: Some("BESPOKE-EXTENSION-ABC".to_string()),
            child_count: None,
        };

        let wildcard_criterion = FilterCriterion {
            target: FilterTarget::Files,
            property: FilterProperty::Extension,
            condition: FilterCondition::Is,
            value: ".BESPOKE-EXTENSION-*".to_string(),
        };
        assert!(evaluate_criterion(&entry, &wildcard_criterion));

        let tmp_entry = FileEntry {
            path: "/tmp/scratch.tmp".to_string(),
            name: "scratch.tmp".to_string(),
            is_dir: false,
            size_bytes: 100,
            drive_usage_percent: 0.01,
            modified_timestamp: None,
            extension: Some("tmp".to_string()),
            child_count: None,
        };

        let tmp_star = FilterCriterion {
            target: FilterTarget::Files,
            property: FilterProperty::Extension,
            condition: FilterCondition::Is,
            value: "*.tmp".to_string(),
        };
        assert!(evaluate_criterion(&tmp_entry, &tmp_star));
    }
}
