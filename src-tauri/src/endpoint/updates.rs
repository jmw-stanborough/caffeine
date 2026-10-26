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

pub fn download_updates(target_id: Option<String>) -> Result<UpdateOperationResult, String> {
    if !cfg!(target_os = "windows") {
        return Err("System updates are only supported on Windows.".to_string());
    }

    let id_arg = target_id.unwrap_or_default();
    let script = format!(
        r#"
$ProgressPreference = 'SilentlyContinue'
$targetId = '{id_arg}'
$Session = New-Object -ComObject Microsoft.Update.Session
$Searcher = $Session.CreateUpdateSearcher()
$SearchResult = $Searcher.Search("IsInstalled=0 and Type='Software' and IsHidden=0")
$Downloader = $Session.CreateUpdateDownloader()
$UpdatesToDownload = New-Object -ComObject Microsoft.Update.UpdateColl
foreach ($u in $SearchResult.Updates) {{
    if (-not $targetId -or $u.Identity.UpdateID -eq $targetId) {{
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
[PSCustomObject]@{{
    Success = $succeeded
    Message = "Download completed with code " + $downloadResult.ResultCode
    SucceededCount = $(if ($succeeded) {{ $UpdatesToDownload.Count }} else {{ 0 }})
    FailedCount = $(if ($succeeded) {{ 0 }} else {{ $UpdatesToDownload.Count }})
    RebootRequired = $reboot
}} | ConvertTo-Json -Compress
"#
    );

    execute_update_operation(&script)
}

pub fn install_updates(target_id: Option<String>) -> Result<UpdateOperationResult, String> {
    if !cfg!(target_os = "windows") {
        return Err("System updates are only supported on Windows.".to_string());
    }

    let id_arg = target_id.unwrap_or_default();
    let script = format!(
        r#"
$ProgressPreference = 'SilentlyContinue'
$targetId = '{id_arg}'
$Session = New-Object -ComObject Microsoft.Update.Session
$Searcher = $Session.CreateUpdateSearcher()
$SearchResult = $Searcher.Search("IsInstalled=0 and Type='Software' and IsHidden=0")
$UpdatesToInstall = New-Object -ComObject Microsoft.Update.UpdateColl
foreach ($u in $SearchResult.Updates) {{
    if ((-not $targetId -or $u.Identity.UpdateID -eq $targetId) -and $u.IsDownloaded) {{
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
[PSCustomObject]@{{
    Success = ($installResult.ResultCode -eq 2)
    Message = "Installation finished with code " + $installResult.ResultCode
    SucceededCount = $(if ($installResult.ResultCode -eq 2) {{ $UpdatesToInstall.Count }} else {{ 0 }})
    FailedCount = $(if ($installResult.ResultCode -eq 2) {{ 0 }} else {{ $UpdatesToInstall.Count }})
    RebootRequired = $reboot
}} | ConvertTo-Json -Compress
"#
    );

    execute_update_operation(&script)
}
