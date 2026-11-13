use super::super::powershell::{
    extract_json_slice, query_powershell_json, run_powershell_streaming,
};
use super::super::types::{UpdateCheckResult, UpdateOperationResult, UpdateProgressEvent};
use super::scripts;
use super::types::{
    format_powershell_id_array, map_raw_update, parse_progress_event, resolve_target_ids,
    RawProgress, RawResult, RawUpdate, RawUpdateOperation,
};

pub(crate) fn check_for_updates() -> Result<UpdateCheckResult, String> {
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

pub(crate) fn download_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    let resolved_ids = resolve_target_ids(target_ids, target_id)?;
    let ids_array = format_powershell_id_array(&resolved_ids);
    let script = scripts::build_download_script(&ids_array);

    execute_streaming_update_operation(&script, on_progress)
}

pub(crate) fn install_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    let resolved_ids = resolve_target_ids(target_ids, target_id)?;
    let ids_array = format_powershell_id_array(&resolved_ids);
    let script = scripts::build_install_script(&ids_array);

    execute_streaming_update_operation(&script, on_progress)
}

pub(crate) fn restart_system() -> Result<(), String> {
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
