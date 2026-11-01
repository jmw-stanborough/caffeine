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
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '{' || c == '}')
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
