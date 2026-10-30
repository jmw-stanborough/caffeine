use super::powershell::query_powershell_json;
use super::types::{SystemUpdateItem, UpdateCheckResult, UpdateOperationResult};

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

fn execute_update_operation(script: &str) -> Result<UpdateOperationResult, String> {
    let raw: RawUpdateOperation = query_powershell_json(script)?;
    Ok(raw.into())
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

pub fn download_updates(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
) -> Result<UpdateOperationResult, String> {
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
$Downloader = $Session.CreateUpdateDownloader()
$UpdatesToDownload = New-Object -ComObject Microsoft.Update.UpdateColl
$filterById = ($targetIds.Count -gt 0)
foreach ($u in $SearchResult.Updates) {{
    if (-not $filterById -or ($targetIds -contains $u.Identity.UpdateID)) {{
        if (-not $u.EulaAccepted) {{
            $u.AcceptEula()
        }}
        $UpdatesToDownload.Add($u) | Out-Null
    }}
}}
if ($UpdatesToDownload.Count -eq 0) {{
    [PSCustomObject]@{{
        Success = $true
        Message = "No updates matched to download."
        SucceededCount = 0
        FailedCount = 0
        RebootRequired = $false
    }} | ConvertTo-Json -Compress
    exit 0
}}
$Downloader.Updates = $UpdatesToDownload
$downloadResult = $Downloader.Download()
$succeeded = ($downloadResult.ResultCode -eq 2)
$reboot = (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired') -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending')
$msg = switch ($downloadResult.ResultCode) {{
    2 {{ "Download completed successfully." }}
    3 {{ "Download completed with errors." }}
    4 {{ "Download failed (code 4). Administrative privileges or a system restart may be required." }}
    5 {{ "Download was aborted (code 5)." }}
    default {{ "Download completed with code " + $downloadResult.ResultCode }}
}}
[PSCustomObject]@{{
    Success = $succeeded
    Message = $msg
    SucceededCount = $(if ($succeeded) {{ $UpdatesToDownload.Count }} else {{ 0 }})
    FailedCount = $(if ($succeeded) {{ 0 }} else {{ $UpdatesToDownload.Count }})
    RebootRequired = $reboot
}} | ConvertTo-Json -Compress
"#
    );

    execute_update_operation(&script)
}

pub fn install_updates(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
) -> Result<UpdateOperationResult, String> {
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
$Installer = $Session.CreateUpdateInstaller()
$Installer.Updates = $UpdatesToInstall
$Installer.ForceQuiet = $true
$installResult = $Installer.Install()
$reboot = [bool]$installResult.RebootRequired -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired') -or (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending')
$succeeded = ($installResult.ResultCode -eq 2)
$msg = switch ($installResult.ResultCode) {{
    2 {{ "Installation completed successfully." }}
    3 {{ "Installation completed with errors." }}
    4 {{ "Installation failed (code 4). Administrative privileges or a system restart may be required." }}
    5 {{ "Installation was aborted (code 5)." }}
    default {{ "Installation finished with code " + $installResult.ResultCode }}
}}
[PSCustomObject]@{{
    Success = $succeeded
    Message = $msg
    SucceededCount = $(if ($succeeded) {{ $UpdatesToInstall.Count }} else {{ 0 }})
    FailedCount = $(if ($succeeded) {{ 0 }} else {{ $UpdatesToInstall.Count }})
    RebootRequired = $reboot
}} | ConvertTo-Json -Compress
"#
    );

    execute_update_operation(&script)
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

    fn expect_eq(actual: Vec<String>, expected: Vec<&str>) {
        let expected_strings: Vec<String> = expected.into_iter().map(String::from).collect();
        assert_eq!(actual, expected_strings);
    }
}
