pub mod scripts;
pub mod types;

use super::powershell::{extract_json_slice, query_powershell_json, run_powershell_streaming};
use super::types::{UpdateCheckResult, UpdateOperationResult, UpdateProgressEvent};
pub use types::{format_powershell_id_array, resolve_target_ids};
use types::{
    map_raw_update, parse_progress_event, RawProgress, RawResult, RawUpdate, RawUpdateOperation,
};

fn ensure_windows_supported() -> Result<(), String> {
    if !cfg!(target_os = "windows") {
        return Err("System updates are only supported on Windows.".to_string());
    }
    Ok(())
}

pub fn check_for_updates() -> Result<UpdateCheckResult, String> {
    ensure_windows_supported()?;

    let script = scripts::build_check_script();
    let parsed: RawResult = query_powershell_json(&script)?;

    let mut update_items = Vec::new();
    if let Some(val) = parsed.updates {
        if let Ok(list) = serde_json::from_value::<Vec<RawUpdate>>(val.clone()) {
            for u in list {
                update_items.push(map_raw_update(u)?);
            }
        } else if let Ok(single) = serde_json::from_value::<RawUpdate>(val.clone()) {
            update_items.push(map_raw_update(single)?);
        } else if !val.is_null()
            && !(val.is_array() && val.as_array().is_some_and(|a| a.is_empty()))
        {
            return Err(format!("Unexpected updates payload format: {val}"));
        }
    }

    let last_checked = parsed
        .last_checked
        .ok_or_else(|| "Missing 'LastChecked' timestamp in update check payload".to_string())?;

    Ok(UpdateCheckResult {
        updates: update_items,
        reboot_required: parsed.reboot_required.unwrap_or(false),
        last_checked,
    })
}

pub fn download_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    ensure_windows_supported()?;

    let resolved_ids = resolve_target_ids(target_ids, target_id)?;
    let ids_array = format_powershell_id_array(&resolved_ids);
    let script = scripts::build_download_script(&ids_array);

    execute_streaming_update_operation(&script, on_progress)
}

pub fn install_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    ensure_windows_supported()?;

    let resolved_ids = resolve_target_ids(target_ids, target_id)?;
    let ids_array = format_powershell_id_array(&resolved_ids);
    let script = scripts::build_install_script(&ids_array);

    execute_streaming_update_operation(&script, on_progress)
}

pub fn restart_system() -> Result<(), String> {
    ensure_windows_supported()?;

    let output = std::process::Command::new("shutdown.exe")
        .args(["/r", "/t", "0", "/d", "p:2:18"])
        .output()
        .map_err(|e| format!("Failed to execute restart command: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("System restart failed: {}", stderr.trim()));
    }

    Ok(())
}

fn execute_streaming_update_operation<F>(
    script: &str,
    mut on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    let mut final_result: Option<UpdateOperationResult> = None;
    let mut stream_err: Option<String> = None;

    run_powershell_streaming(script, |line| {
        if stream_err.is_some() {
            return;
        }

        if let Ok(slice) = extract_json_slice(line) {
            if let Ok(raw_progress) = serde_json::from_str::<RawProgress>(slice) {
                match parse_progress_event(raw_progress) {
                    Ok(Some(event)) => {
                        on_progress(event);
                        return;
                    }
                    Err(err) => {
                        stream_err = Some(err);
                        return;
                    }
                    Ok(None) => {}
                }
            }

            if let Ok(raw_op) = serde_json::from_str::<RawUpdateOperation>(slice) {
                match UpdateOperationResult::try_from(raw_op) {
                    Ok(op) => {
                        final_result = Some(op);
                    }
                    Err(err) => {
                        stream_err = Some(err);
                    }
                }
            }
        }
    })?;

    if let Some(err) = stream_err {
        return Err(err);
    }

    final_result.ok_or_else(|| {
        "PowerShell update process completed without producing an operation result.".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target_ids_merges_and_sanitizes() {
        let list = Some(vec!["update-1".to_string(), "update-2".to_string()]);
        let single = Some("update-3".to_string());
        let resolved = resolve_target_ids(list, single).unwrap();
        expect_eq(resolved, vec!["update-1", "update-2", "update-3"]);
    }

    #[test]
    fn test_resolve_target_ids_rejects_special_characters() {
        let malformed = Some(vec!["update; Remove-Item C:\\".to_string()]);
        assert!(resolve_target_ids(malformed, None).is_err());
    }

    #[test]
    fn test_format_powershell_id_array_empty_and_populated() {
        assert_eq!(format_powershell_id_array(&[]), "@()");
        assert_eq!(
            format_powershell_id_array(&["id1".to_string(), "id2".to_string()]),
            "@('id1', 'id2')"
        );
    }

    #[test]
    fn test_raw_progress_deserialization() {
        let json = r#"{"Type":"progress","Phase":"download","Percent":65,"CurrentIndex":1,"TotalCount":3,"BytesDownloaded":1048576,"TotalBytes":2097152,"CurrentUpdateId":"uuid-123"}"#;
        let raw: RawProgress = serde_json::from_str(json).unwrap();
        assert_eq!(raw.msg_type.as_deref(), Some("progress"));
        assert_eq!(raw.phase.as_deref(), Some("download"));
        assert_eq!(raw.percent, Some(65));
        assert_eq!(raw.current_index, Some(1));
        assert_eq!(raw.total_count, Some(3));
        assert_eq!(raw.bytes_downloaded, Some(1048576));
        assert_eq!(raw.total_bytes, Some(2097152));
        assert_eq!(raw.current_update_id.as_deref(), Some("uuid-123"));
    }

    #[test]
    fn test_raw_update_operation_deserialization() {
        let json = r#"{"Success":true,"Message":"Installed","SucceededCount":2,"FailedCount":0,"RebootRequired":true}"#;
        let raw: RawUpdateOperation = serde_json::from_str(json).unwrap();
        let res = UpdateOperationResult::try_from(raw).unwrap();
        assert!(res.success);
        assert!(res.reboot_required);
        assert_eq!(res.succeeded_count, 2);
        assert_eq!(res.failed_count, 0);
        assert_eq!(res.message, "Installed");
    }

    #[test]
    fn test_raw_update_operation_missing_success_fails() {
        let json = r#"{"Message":"Installed","SucceededCount":1}"#;
        let raw: RawUpdateOperation = serde_json::from_str(json).unwrap();
        assert!(UpdateOperationResult::try_from(raw).is_err());
    }

    fn expect_eq(actual: Vec<String>, expected: Vec<&str>) {
        let expected_strings: Vec<String> = expected.into_iter().map(String::from).collect();
        assert_eq!(actual, expected_strings);
    }
}
