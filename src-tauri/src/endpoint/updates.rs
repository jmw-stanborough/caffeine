use super::powershell::{extract_json_slice, query_powershell_json, run_powershell_streaming};
use super::types::{
    SystemUpdateItem, UpdateCheckResult, UpdateOperationResult, UpdateProgressEvent,
};

#[derive(serde::Deserialize)]
struct RawUpdate {
    #[serde(rename = "Id")]
    id: Option<String>,
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "KB")]
    kb: Option<String>,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "Size")]
    size: Option<u64>,
    #[serde(rename = "Categories")]
    categories: Option<Vec<String>>,
    #[serde(rename = "Severity")]
    severity: Option<String>,
    #[serde(rename = "IsMandatory")]
    is_mandatory: Option<bool>,
    #[serde(rename = "IsDownloaded")]
    is_downloaded: Option<bool>,
    #[serde(rename = "SupportUrl")]
    support_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawResult {
    #[serde(rename = "Updates")]
    updates: Option<serde_json::Value>,
    #[serde(rename = "RebootRequired")]
    reboot_required: Option<bool>,
    #[serde(rename = "LastChecked")]
    last_checked: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawUpdateOperation {
    #[serde(rename = "Success")]
    success: Option<bool>,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "SucceededCount")]
    succeeded_count: Option<usize>,
    #[serde(rename = "FailedCount")]
    failed_count: Option<usize>,
    #[serde(rename = "RebootRequired")]
    reboot_required: Option<bool>,
}

impl From<RawUpdateOperation> for UpdateOperationResult {
    fn from(raw: RawUpdateOperation) -> Self {
        Self {
            success: raw.success.unwrap_or(false),
            reboot_required: raw.reboot_required.unwrap_or(false),
            message: raw.message.unwrap_or_default(),
            succeeded_count: raw.succeeded_count.unwrap_or(0),
            failed_count: raw.failed_count.unwrap_or(0),
        }
    }
}

pub fn check_for_updates() -> Result<UpdateCheckResult, String> {
    if !cfg!(target_os = "windows") {
        return Err("System updates are only supported on Windows.".to_string());
    }

    let script = r#"
$ProgressPreference = 'SilentlyContinue'
$Session = New-Object -ComObject Microsoft.Update.Session
$Searcher = $Session.CreateUpdateSearcher()
$SearchResult = $Searcher.Search("IsInstalled=0 and Type='Software' and IsHidden=0")
$Updates = @()
foreach ($u in $SearchResult.Updates) {
    $kb = $null
    if ($u.KBArticleIDs.Count -gt 0) {
        $kb = "KB" + $u.KBArticleIDs.Item(0)
    }
    $cats = @()
    foreach ($c in $u.Categories) {
        $cats += $c.Name
    }
    $Updates += [PSCustomObject]@{
        Id = $u.Identity.UpdateID
        Title = $u.Title
        KB = $kb
        Description = $u.Description
        Size = $u.MaxDownloadSize
        Categories = $cats
        Severity = [string]$u.MsrcSeverity
        IsMandatory = [bool]$u.IsMandatory
        IsDownloaded = [bool]$u.IsDownloaded
        SupportUrl = $u.SupportUrl
    }
}
$rebootPending = (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired') -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending')
[PSCustomObject]@{
    Updates = $Updates
    RebootRequired = $rebootPending
    LastChecked = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss')
} | ConvertTo-Json -Depth 4 -Compress
"#;

    let parsed: RawResult = query_powershell_json(script)?;

    let mut update_items = Vec::new();
    if let Some(val) = parsed.updates {
        if let Ok(list) = serde_json::from_value::<Vec<RawUpdate>>(val.clone()) {
            for u in list {
                update_items.push(map_raw_update(u));
            }
        } else if let Ok(single) = serde_json::from_value::<RawUpdate>(val) {
            update_items.push(map_raw_update(single));
        }
    }

    Ok(UpdateCheckResult {
        updates: update_items,
        reboot_required: parsed.reboot_required.unwrap_or(false),
        last_checked: parsed.last_checked.unwrap_or_else(|| "Just now".to_string()),
    })
}

fn map_raw_update(raw: RawUpdate) -> SystemUpdateItem {
    SystemUpdateItem {
        id: raw.id.unwrap_or_default(),
        title: raw.title.unwrap_or_else(|| "Unknown Update".to_string()),
        kb_article_id: raw.kb,
        description: raw.description,
        size_bytes: raw.size,
        categories: raw.categories.unwrap_or_default(),
        severity: raw.severity,
        is_mandatory: raw.is_mandatory.unwrap_or(false),
        is_downloaded: raw.is_downloaded.unwrap_or(false),
        support_url: raw.support_url,
    }
}

#[derive(serde::Deserialize)]
struct RawProgress {
    #[serde(rename = "Type")]
    msg_type: Option<String>,
    #[serde(rename = "Phase")]
    phase: Option<String>,
    #[serde(rename = "Percent")]
    percent: Option<u32>,
    #[serde(rename = "CurrentIndex")]
    current_index: Option<usize>,
    #[serde(rename = "TotalCount")]
    total_count: Option<usize>,
    #[serde(rename = "BytesDownloaded")]
    bytes_downloaded: Option<u64>,
    #[serde(rename = "TotalBytes")]
    total_bytes: Option<u64>,
    #[serde(rename = "CurrentUpdateId")]
    current_update_id: Option<String>,
}

fn execute_streaming_update_operation<F>(
    script: &str,
    mut on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    let mut final_result: Option<UpdateOperationResult> = None;

    run_powershell_streaming(script, |line| {
        if let Ok(slice) = extract_json_slice(line) {
            if let Ok(progress) = serde_json::from_str::<RawProgress>(slice) {
                if progress.msg_type.as_deref() == Some("progress") {
                    on_progress(UpdateProgressEvent {
                        phase: progress.phase.unwrap_or_else(|| "download".to_string()),
                        percent_complete: progress.percent.unwrap_or(0),
                        current_index: progress.current_index.unwrap_or(0),
                        total_count: progress.total_count.unwrap_or(0),
                        bytes_downloaded: progress.bytes_downloaded,
                        total_bytes: progress.total_bytes,
                        current_update_id: progress.current_update_id,
                    });
                    return;
                }
            }

            if let Ok(raw_op) = serde_json::from_str::<RawUpdateOperation>(slice) {
                if raw_op.success.is_some() {
                    final_result = Some(raw_op.into());
                }
            }
        }
    })?;

    final_result.ok_or_else(|| {
        "PowerShell update process completed without producing an operation result.".to_string()
    })
}

pub(crate) fn resolve_target_ids(
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
        if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '{' || c == '}') {
            return Err(format!("Invalid update ID format: {id}"));
        }
    }

    Ok(result)
}

pub(crate) fn format_powershell_id_array(ids: &[String]) -> String {
    if ids.is_empty() {
        "@()".to_string()
    } else {
        let quoted: Vec<String> = ids.iter().map(|id| format!("'{id}'")).collect();
        format!("@({})", quoted.join(", "))
    }
}

pub fn download_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    if !cfg!(target_os = "windows") {
        return Err("System updates are only supported on Windows.".to_string());
    }

    let resolved_ids = resolve_target_ids(target_ids, target_id)?;
    let ids_array = format_powershell_id_array(&resolved_ids);

    let script = format!(
        r#"
$ProgressPreference = 'SilentlyContinue'
$targetIds = {ids_array}
$Session = New-Object -ComObject Microsoft.Update.Session
$Searcher = $Session.CreateUpdateSearcher()
$SearchResult = $Searcher.Search("IsInstalled=0 and Type='Software' and IsHidden=0")
$UpdatesToDownload = New-Object -ComObject Microsoft.Update.UpdateColl
$filterById = ($targetIds.Count -gt 0)
foreach ($u in $SearchResult.Updates) {{
    if ((-not $filterById -or ($targetIds -contains $u.Identity.UpdateID)) -and (-not $u.IsDownloaded)) {{
        if (-not $u.EulaAccepted) {{
            $u.AcceptEula()
        }}
        $UpdatesToDownload.Add($u) | Out-Null
    }}
}}
if ($UpdatesToDownload.Count -eq 0) {{
    [PSCustomObject]@{{
        Success = $true
        Message = "All selected updates are already downloaded."
        SucceededCount = 0
        FailedCount = 0
        RebootRequired = $false
    }} | ConvertTo-Json -Compress
    exit 0
}}

$totalCount = $UpdatesToDownload.Count
$totalBytes = 0
for ($i = 0; $i -lt $totalCount; $i++) {{
    $totalBytes += $UpdatesToDownload.Item($i).MaxDownloadSize
}}

$accumulatedBytes = 0
$succeededCount = 0
$failedCount = 0

for ($i = 0; $i -lt $totalCount; $i++) {{
    $currentUpdate = $UpdatesToDownload.Item($i)
    $currId = $currentUpdate.Identity.UpdateID
    $currBytes = $currentUpdate.MaxDownloadSize

    $pct = [int][math]::Floor(($i / $totalCount) * 100)
    [PSCustomObject]@{{
        Type = "progress"
        Phase = "download"
        Percent = $pct
        CurrentIndex = $i
        TotalCount = $totalCount
        BytesDownloaded = $accumulatedBytes
        TotalBytes = $totalBytes
        CurrentUpdateId = $currId
    }} | ConvertTo-Json -Compress

    $singleColl = New-Object -ComObject Microsoft.Update.UpdateColl
    $singleColl.Add($currentUpdate) | Out-Null
    $singleDownloader = $Session.CreateUpdateDownloader()
    $singleDownloader.Updates = $singleColl

    try {{
        $res = $singleDownloader.Download()
        if ($res.ResultCode -eq 2) {{
            $succeededCount++
            $accumulatedBytes += $currBytes
        }} else {{
            $failedCount++
        }}
    }} catch {{
        $failedCount++
    }}

    $pctAfter = [int][math]::Floor((($i + 1) / $totalCount) * 100)
    [PSCustomObject]@{{
        Type = "progress"
        Phase = "download"
        Percent = $pctAfter
        CurrentIndex = $i
        TotalCount = $totalCount
        BytesDownloaded = $accumulatedBytes
        TotalBytes = $totalBytes
        CurrentUpdateId = $currId
    }} | ConvertTo-Json -Compress
}}

$reboot = (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired') -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending')
$succeeded = ($failedCount -eq 0 -and $succeededCount -gt 0)
$msg = if ($succeeded) {{
    "Download completed successfully."
}} elseif ($succeededCount -gt 0) {{
    "Download completed with errors ($succeededCount succeeded, $failedCount failed)."
}} else {{
    "Download failed. Administrative privileges or a system restart may be required."
}}

[PSCustomObject]@{{
    Success = $succeeded
    Message = $msg
    SucceededCount = $succeededCount
    FailedCount = $failedCount
    RebootRequired = $reboot
}} | ConvertTo-Json -Compress
"#
    );

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
    if !cfg!(target_os = "windows") {
        return Err("System updates are only supported on Windows.".to_string());
    }

    let resolved_ids = resolve_target_ids(target_ids, target_id)?;
    let ids_array = format_powershell_id_array(&resolved_ids);

    let script = format!(
        r#"
$ProgressPreference = 'SilentlyContinue'
$rebootPending = (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired') -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending')
if ($rebootPending) {{
    [PSCustomObject]@{{
        Success = $false
        Message = "A system restart is required before installing further updates. Please restart your system and try again."
        SucceededCount = 0
        FailedCount = 0
        RebootRequired = $true
    }} | ConvertTo-Json -Compress
    exit 0
}}

$targetIds = {ids_array}
$Session = New-Object -ComObject Microsoft.Update.Session
$Searcher = $Session.CreateUpdateSearcher()
$SearchResult = $Searcher.Search("IsInstalled=0 and Type='Software' and IsHidden=0")
$UpdatesToInstall = New-Object -ComObject Microsoft.Update.UpdateColl
$filterById = ($targetIds.Count -gt 0)
foreach ($u in $SearchResult.Updates) {{
    if ((-not $filterById -or ($targetIds -contains $u.Identity.UpdateID)) -and $u.IsDownloaded) {{
        if (-not $u.EulaAccepted) {{
            $u.AcceptEula()
        }}
        $UpdatesToInstall.Add($u) | Out-Null
    }}
}}
if ($UpdatesToInstall.Count -eq 0) {{
    [PSCustomObject]@{{
        Success = $false
        Message = "No downloaded updates found ready for installation. Please download them first."
        SucceededCount = 0
        FailedCount = 0
        RebootRequired = $false
    }} | ConvertTo-Json -Compress
    exit 0
}}

$totalCount = $UpdatesToInstall.Count
$succeededCount = 0
$failedCount = 0
$rebootNeeded = $false

for ($i = 0; $i -lt $totalCount; $i++) {{
    $currentUpdate = $UpdatesToInstall.Item($i)
    $currId = $currentUpdate.Identity.UpdateID

    $pct = [int][math]::Floor(($i / $totalCount) * 100)
    [PSCustomObject]@{{
        Type = "progress"
        Phase = "install"
        Percent = $pct
        CurrentIndex = $i
        TotalCount = $totalCount
        BytesDownloaded = $null
        TotalBytes = $null
        CurrentUpdateId = $currId
    }} | ConvertTo-Json -Compress

    $singleColl = New-Object -ComObject Microsoft.Update.UpdateColl
    $singleColl.Add($currentUpdate) | Out-Null
    $singleInstaller = $Session.CreateUpdateInstaller()
    $singleInstaller.Updates = $singleColl
    $singleInstaller.ForceQuiet = $true

    try {{
        $res = $singleInstaller.Install()
        if ($res.ResultCode -eq 2) {{
            $succeededCount++
        }} else {{
            $failedCount++
        }}
        if ($res.RebootRequired) {{
            $rebootNeeded = $true
        }}
    }} catch {{
        $failedCount++
    }}

    $pctAfter = [int][math]::Floor((($i + 1) / $totalCount) * 100)
    [PSCustomObject]@{{
        Type = "progress"
        Phase = "install"
        Percent = $pctAfter
        CurrentIndex = $i
        TotalCount = $totalCount
        BytesDownloaded = $null
        TotalBytes = $null
        CurrentUpdateId = $currId
    }} | ConvertTo-Json -Compress
}}

$reboot = $rebootNeeded -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired') -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending')
$succeeded = ($failedCount -eq 0 -and $succeededCount -gt 0)
$msg = if ($succeeded) {{
    "Installation completed successfully."
}} elseif ($succeededCount -gt 0) {{
    "Installation completed with errors ($succeededCount succeeded, $failedCount failed)."
}} else {{
    "Installation failed. Administrative privileges or a system restart may be required."
}}

[PSCustomObject]@{{
    Success = $succeeded
    Message = $msg
    SucceededCount = $succeededCount
    FailedCount = $failedCount
    RebootRequired = $reboot
}} | ConvertTo-Json -Compress
"#
    );

    execute_streaming_update_operation(&script, on_progress)
}

pub fn restart_system() -> Result<(), String> {
    if !cfg!(target_os = "windows") {
        return Err("System restart is only supported on Windows.".to_string());
    }

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
        let res: UpdateOperationResult = raw.into();
        assert!(res.success);
        assert!(res.reboot_required);
        assert_eq!(res.succeeded_count, 2);
        assert_eq!(res.failed_count, 0);
        assert_eq!(res.message, "Installed");
    }

    fn expect_eq(actual: Vec<String>, expected: Vec<&str>) {
        let expected_strings: Vec<String> = expected.into_iter().map(String::from).collect();
        assert_eq!(actual, expected_strings);
    }
}
