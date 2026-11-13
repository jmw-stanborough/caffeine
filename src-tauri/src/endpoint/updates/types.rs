use super::super::types::{SystemUpdateItem, UpdateOperationResult, UpdateProgressEvent};

#[derive(serde::Deserialize)]
pub(crate) struct RawUpdate {
    #[serde(rename = "Id")]
    pub id: Option<String>,
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "KB")]
    pub kb: Option<String>,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Size")]
    pub size: Option<u64>,
    #[serde(rename = "Categories")]
    pub categories: Option<Vec<String>>,
    #[serde(rename = "Severity")]
    pub severity: Option<String>,
    #[serde(rename = "IsMandatory")]
    pub is_mandatory: Option<bool>,
    #[serde(rename = "IsDownloaded")]
    pub is_downloaded: Option<bool>,
    #[serde(rename = "SupportUrl")]
    pub support_url: Option<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct RawResult {
    #[serde(rename = "Updates")]
    pub updates: Option<serde_json::Value>,
    #[serde(rename = "RebootRequired")]
    pub reboot_required: Option<bool>,
    #[serde(rename = "LastChecked")]
    pub last_checked: Option<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct RawUpdateOperation {
    #[serde(rename = "Success")]
    pub success: Option<bool>,
    #[serde(rename = "Message")]
    pub message: Option<String>,
    #[serde(rename = "SucceededCount")]
    pub succeeded_count: Option<usize>,
    #[serde(rename = "FailedCount")]
    pub failed_count: Option<usize>,
    #[serde(rename = "RebootRequired")]
    pub reboot_required: Option<bool>,
}

impl TryFrom<RawUpdateOperation> for UpdateOperationResult {
    type Error = String;

    fn try_from(raw: RawUpdateOperation) -> Result<Self, Self::Error> {
        let success = raw
            .success
            .ok_or_else(|| "Missing 'Success' field in update operation payload".to_string())?;

        Ok(UpdateOperationResult {
            success,
            reboot_required: raw.reboot_required.unwrap_or(false),
            message: raw.message.unwrap_or_default(),
            succeeded_count: raw.succeeded_count.unwrap_or(0),
            failed_count: raw.failed_count.unwrap_or(0),
        })
    }
}

pub(crate) fn map_raw_update(raw: RawUpdate) -> Result<SystemUpdateItem, String> {
    let id = raw
        .id
        .ok_or_else(|| "Update item is missing mandatory 'Id' identifier".to_string())?;
    let title = raw
        .title
        .ok_or_else(|| format!("Update '{id}' is missing mandatory 'Title'"))?;

    Ok(SystemUpdateItem {
        id,
        title,
        kb_article_id: raw.kb,
        description: raw.description,
        size_bytes: raw.size,
        categories: raw.categories.unwrap_or_default(),
        severity: raw.severity,
        is_mandatory: raw.is_mandatory.unwrap_or(false),
        is_downloaded: raw.is_downloaded.unwrap_or(false),
        support_url: raw.support_url,
    })
}

#[derive(serde::Deserialize)]
pub(crate) struct RawProgress {
    #[serde(rename = "Type")]
    pub msg_type: Option<String>,
    #[serde(rename = "Phase")]
    pub phase: Option<String>,
    #[serde(rename = "Percent")]
    pub percent: Option<u32>,
    #[serde(rename = "CurrentIndex")]
    pub current_index: Option<usize>,
    #[serde(rename = "TotalCount")]
    pub total_count: Option<usize>,
    #[serde(rename = "BytesDownloaded")]
    pub bytes_downloaded: Option<u64>,
    #[serde(rename = "TotalBytes")]
    pub total_bytes: Option<u64>,
    #[serde(rename = "CurrentUpdateId")]
    pub current_update_id: Option<String>,
}

pub(crate) fn parse_progress_event(raw: RawProgress) -> Result<Option<UpdateProgressEvent>, String> {
    if raw.msg_type.as_deref() != Some("progress") {
        return Ok(None);
    }

    let phase = raw
        .phase
        .ok_or_else(|| "Missing 'Phase' in progress payload".to_string())?;

    Ok(Some(UpdateProgressEvent {
        phase,
        percent_complete: raw.percent.unwrap_or(0),
        current_index: raw.current_index.unwrap_or(0),
        total_count: raw.total_count.unwrap_or(0),
        bytes_downloaded: raw.bytes_downloaded,
        total_bytes: raw.total_bytes,
        current_update_id: raw.current_update_id,
    }))
}

pub fn resolve_target_ids(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
) -> Result<Vec<String>, String> {
    let mut result = target_ids.unwrap_or_default();
    if let Some(single) = target_id {
        let trimmed = single.trim().to_string();
        if !trimmed.is_empty() && !result.contains(&trimmed) {
            result.push(trimmed);
        }
    }

    for id in &result {
        if id.is_empty()
            || !id.chars().all(|c| {
                c.is_ascii_alphanumeric()
                    || matches!(c, '-' | '_' | '{' | '}' | '.' | '+' | ':' | '~' | '@')
            })
        {
            return Err(format!("Invalid update ID format: {id}"));
        }
    }

    Ok(result)
}

pub fn format_powershell_id_array(ids: &[String]) -> String {
    if ids.is_empty() {
        "@()".to_string()
    } else {
        let quoted: Vec<String> = ids.iter().map(|id| format!("'{id}'")).collect();
        format!("@({})", quoted.join(", "))
    }
}

#[cfg(any(unix, test))]
pub(crate) fn current_utc_timestamp() -> Result<String, String> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("System clock is before Unix epoch: {e}"))?
        .as_secs();

    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let min = (rem % 3_600) / 60;
    let sec = rem % 60;

    // Howard Hinnant's civil-from-days algorithm avoids external date crates.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    Ok(format!(
        "{year:04}-{m:02}-{d:02} {hour:02}:{min:02}:{sec:02} UTC"
    ))
}

#[cfg(unix)]
pub(crate) fn format_command_failure(context: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return format!("{context}: {stderr}");
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return format!("{context}: {stdout}");
    }
    format!("{context} exited with status {:?}", output.status.code())
}

#[cfg(any(unix, test))]
pub(crate) fn run_package_batch<F, Op, R>(
    phase: &str,
    targets: Vec<String>,
    empty_message: &str,
    mut on_progress: F,
    mut op: Op,
    check_reboot: R,
) -> UpdateOperationResult
where
    F: FnMut(UpdateProgressEvent),
    Op: FnMut(&str) -> Result<(), String>,
    R: FnOnce() -> bool,
{
    let total_count = targets.len();
    if total_count == 0 {
        return UpdateOperationResult {
            success: true,
            reboot_required: check_reboot(),
            message: empty_message.to_string(),
            succeeded_count: 0,
            failed_count: 0,
        };
    }

    let mut succeeded_count = 0usize;
    let mut failed_count = 0usize;
    let mut last_error: Option<String> = None;

    for (index, target) in targets.iter().enumerate() {
        let pct_before = ((index as u64 * 100) / total_count as u64) as u32;
        on_progress(UpdateProgressEvent {
            phase: phase.to_string(),
            percent_complete: pct_before,
            current_index: index,
            total_count,
            bytes_downloaded: None,
            total_bytes: None,
            current_update_id: Some(target.clone()),
        });

        match op(target) {
            Ok(()) => succeeded_count += 1,
            Err(err) => {
                failed_count += 1;
                last_error = Some(err);
            }
        }

        let pct_after = (((index + 1) as u64 * 100) / total_count as u64) as u32;
        on_progress(UpdateProgressEvent {
            phase: phase.to_string(),
            percent_complete: pct_after,
            current_index: index,
            total_count,
            bytes_downloaded: None,
            total_bytes: None,
            current_update_id: Some(target.clone()),
        });
    }

    let action_label = if phase == "download" {
        "Download"
    } else {
        "Installation"
    };
    let success = failed_count == 0 && succeeded_count > 0;
    let message = if success {
        format!("{action_label} completed successfully.")
    } else if succeeded_count > 0 {
        match last_error {
            Some(err) => format!(
                "{action_label} completed with errors ({succeeded_count} succeeded, {failed_count} failed): {err}"
            ),
            None => format!(
                "{action_label} completed with errors ({succeeded_count} succeeded, {failed_count} failed)."
            ),
        }
    } else {
        match last_error {
            Some(err) => format!("{action_label} failed: {err}"),
            None => format!("{action_label} failed."),
        }
    };

    UpdateOperationResult {
        success,
        reboot_required: check_reboot(),
        message,
        succeeded_count,
        failed_count,
    }
}
