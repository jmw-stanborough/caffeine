#[cfg(any(target_os = "macos", test))]
use super::super::types::SystemUpdateItem;
#[cfg(target_os = "macos")]
use super::super::types::{UpdateCheckResult, UpdateOperationResult, UpdateProgressEvent};
#[cfg(target_os = "macos")]
use super::types::{
    current_utc_timestamp, format_command_failure, resolve_target_ids, run_package_batch,
};

#[cfg(any(target_os = "macos", test))]
fn parse_size_bytes(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    let lower = trimmed.to_ascii_lowercase();
    let (num_str, multiplier) = if let Some(s) = lower.strip_suffix("gib") {
        (s, 1024_f64 * 1024_f64 * 1024_f64)
    } else if let Some(s) = lower.strip_suffix("gb") {
        (s, 1024_f64 * 1024_f64 * 1024_f64)
    } else if let Some(s) = lower.strip_suffix('g') {
        (s, 1024_f64 * 1024_f64 * 1024_f64)
    } else if let Some(s) = lower.strip_suffix("mib") {
        (s, 1024_f64 * 1024_f64)
    } else if let Some(s) = lower.strip_suffix("mb") {
        (s, 1024_f64 * 1024_f64)
    } else if let Some(s) = lower.strip_suffix('m') {
        (s, 1024_f64 * 1024_f64)
    } else if let Some(s) = lower.strip_suffix("kib") {
        (s, 1024_f64)
    } else if let Some(s) = lower.strip_suffix("kb") {
        (s, 1024_f64)
    } else if let Some(s) = lower.strip_suffix('k') {
        (s, 1024_f64)
    } else if let Some(s) = lower.strip_suffix('b') {
        (s, 1.0)
    } else {
        (lower.as_str(), 1.0)
    };

    let value: f64 = num_str.trim().parse().ok()?;
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    Some((value * multiplier).round() as u64)
}

#[cfg(any(target_os = "macos", test))]
#[derive(Default)]
struct PendingMacUpdate {
    label: String,
    title: Option<String>,
    version: Option<String>,
    size_bytes: Option<u64>,
    recommended: bool,
    requires_restart: bool,
}

#[cfg(any(target_os = "macos", test))]
impl PendingMacUpdate {
    fn into_item(self) -> SystemUpdateItem {
        let title = self.title.unwrap_or_else(|| self.label.clone());
        let mut desc_parts = Vec::new();
        if let Some(ver) = self.version {
            desc_parts.push(format!("Version: {ver}"));
        }
        if self.requires_restart {
            desc_parts.push("Action: restart".to_string());
        }
        let description = (!desc_parts.is_empty()).then(|| desc_parts.join(", "));
        let categories = if self.recommended {
            vec!["Recommended".to_string()]
        } else {
            vec!["Software Update".to_string()]
        };

        SystemUpdateItem {
            id: self.label,
            title,
            kb_article_id: None,
            description,
            size_bytes: self.size_bytes,
            categories,
            severity: self.recommended.then(|| "Important".to_string()),
            is_mandatory: self.recommended,
            is_downloaded: false,
            support_url: None,
        }
    }
}

#[cfg(any(target_os = "macos", test))]
pub(crate) fn parse_softwareupdate_list(stdout: &str) -> (Vec<SystemUpdateItem>, bool) {
    let mut items = Vec::new();
    let mut any_restart_required = false;
    let mut current: Option<PendingMacUpdate> = None;

    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(label) = line.strip_prefix("* Label:") {
            if let Some(pending) = current.take() {
                if pending.requires_restart {
                    any_restart_required = true;
                }
                items.push(pending.into_item());
            }
            let trimmed_label = label.trim();
            if !trimmed_label.is_empty() {
                current = Some(PendingMacUpdate {
                    label: trimmed_label.to_string(),
                    ..Default::default()
                });
            }
            continue;
        }

        let Some(pending) = current.as_mut() else {
            continue;
        };

        for segment in line.split(',') {
            let part = segment.trim();
            if part.is_empty() {
                continue;
            }
            if let Some(val) = part.strip_prefix("Title:") {
                let v = val.trim();
                if !v.is_empty() {
                    pending.title = Some(v.to_string());
                }
            } else if let Some(val) = part.strip_prefix("Version:") {
                let v = val.trim();
                if !v.is_empty() {
                    pending.version = Some(v.to_string());
                }
            } else if let Some(val) = part.strip_prefix("Size:") {
                pending.size_bytes = parse_size_bytes(val);
            } else if let Some(val) = part.strip_prefix("Recommended:") {
                pending.recommended = val.trim().eq_ignore_ascii_case("YES");
            } else if let Some(val) = part.strip_prefix("Action:") {
                pending.requires_restart = val.trim().to_ascii_lowercase().contains("restart");
            } else if pending.version.is_none() {
                // Rejoin segments when a Title value itself contains a comma.
                if let Some(ref mut title) = pending.title {
                    title.push_str(", ");
                    title.push_str(part);
                }
            }
        }
    }

    if let Some(pending) = current.take() {
        if pending.requires_restart {
            any_restart_required = true;
        }
        items.push(pending.into_item());
    }

    (items, any_restart_required)
}

#[cfg(target_os = "macos")]
fn query_softwareupdate() -> Result<(Vec<SystemUpdateItem>, bool), String> {
    let output = std::process::Command::new("softwareupdate")
        .arg("-l")
        .output()
        .map_err(|e| format!("Failed to execute softwareupdate -l: {e}"))?;

    if !output.status.success() {
        return Err(format_command_failure("softwareupdate -l", &output));
    }

    Ok(parse_softwareupdate_list(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

#[cfg(target_os = "macos")]
pub(crate) fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let (updates, reboot_required) = query_softwareupdate()?;
    let last_checked = current_utc_timestamp()?;

    Ok(UpdateCheckResult {
        updates,
        reboot_required,
        last_checked,
    })
}

#[cfg(target_os = "macos")]
fn execute_macos_update_operation<F>(
    phase: &str,
    flag: &str,
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    let mut targets = resolve_target_ids(target_ids, target_id)?;
    let needs_reboot = std::cell::Cell::new(false);

    if targets.is_empty() {
        let (items, list_reboot) = query_softwareupdate()?;
        needs_reboot.set(list_reboot);
        targets = items.into_iter().map(|u| u.id).collect();
    }

    let empty_message = if flag == "-d" {
        "All selected updates are already downloaded."
    } else {
        "No updates available to install."
    };

    Ok(run_package_batch(
        phase,
        targets,
        empty_message,
        on_progress,
        |label| {
            let output = std::process::Command::new("softwareupdate")
                .args([flag, label])
                .output()
                .map_err(|e| format!("Failed to execute softwareupdate {flag} '{label}': {e}"))?;

            if !output.status.success() {
                return Err(format_command_failure(
                    &format!("softwareupdate {flag} '{label}'"),
                    &output,
                ));
            }

            if flag == "-i" {
                let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
                if stdout.contains("restart") || stdout.contains("reboot") {
                    needs_reboot.set(true);
                }
            }

            Ok(())
        },
        || needs_reboot.get(),
    ))
}

#[cfg(target_os = "macos")]
pub(crate) fn download_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    execute_macos_update_operation("download", "-d", target_ids, target_id, on_progress)
}

#[cfg(target_os = "macos")]
pub(crate) fn install_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    execute_macos_update_operation("install", "-i", target_ids, target_id, on_progress)
}

#[cfg(target_os = "macos")]
pub(crate) fn restart_system() -> Result<(), String> {
    let output = std::process::Command::new("shutdown")
        .args(["-r", "now"])
        .output()
        .map_err(|e| format!("Failed to execute restart command: {e}"))?;

    if !output.status.success() {
        return Err(format_command_failure("shutdown -r now", &output));
    }

    Ok(())
}
