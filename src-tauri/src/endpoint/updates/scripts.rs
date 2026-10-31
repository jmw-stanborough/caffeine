pub const REBOOT_PENDING_EXPR: &str = "(Test-Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\WindowsUpdate\\Auto Update\\RebootRequired') -or (Test-Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Component Based Servicing\\RebootPending')";

pub fn build_check_script() -> String {
    format!(
        r#"
$ProgressPreference = 'SilentlyContinue'
$Session = New-Object -ComObject Microsoft.Update.Session
$Searcher = $Session.CreateUpdateSearcher()
$SearchResult = $Searcher.Search("IsInstalled=0 and Type='Software' and IsHidden=0")
$Updates = @()
foreach ($u in $SearchResult.Updates) {{
    $kb = $null
    if ($u.KBArticleIDs.Count -gt 0) {{
        $kb = "KB" + $u.KBArticleIDs.Item(0)
    }}
    $cats = @()
    foreach ($c in $u.Categories) {{
        $cats += $c.Name
    }}
    $Updates += [PSCustomObject]@{{
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
    }}
}}
$rebootPending = {REBOOT_PENDING_EXPR}
[PSCustomObject]@{{
    Updates = $Updates
    RebootRequired = $rebootPending
    LastChecked = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss')
}} | ConvertTo-Json -Depth 4 -Compress
"#
    )
}

pub fn build_download_script(ids_array: &str) -> String {
    format!(
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
$lastError = $null

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
            $lastError = "Download failed with result code $($res.ResultCode)"
        }}
    }} catch {{
        $failedCount++
        $lastError = $_.Exception.Message
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

$reboot = {REBOOT_PENDING_EXPR}
$succeeded = ($failedCount -eq 0 -and $succeededCount -gt 0)
$msg = if ($succeeded) {{
    "Download completed successfully."
}} elseif ($succeededCount -gt 0) {{
    if ($lastError) {{
        "Download completed with errors ($succeededCount succeeded, $failedCount failed): $lastError"
    }} else {{
        "Download completed with errors ($succeededCount succeeded, $failedCount failed)."
    }}
}} else {{
    if ($lastError) {{
        "Download failed: $lastError"
    }} else {{
        "Download failed. Administrative privileges or a system restart may be required."
    }}
}}

[PSCustomObject]@{{
    Success = $succeeded
    Message = $msg
    SucceededCount = $succeededCount
    FailedCount = $failedCount
    RebootRequired = $reboot
}} | ConvertTo-Json -Compress
"#
    )
}

pub fn build_install_script(ids_array: &str) -> String {
    format!(
        r#"
$ProgressPreference = 'SilentlyContinue'
$rebootPending = {REBOOT_PENDING_EXPR}
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
$lastError = $null

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
            $lastError = "Installation failed with result code $($res.ResultCode)"
        }}
        if ($res.RebootRequired) {{
            $rebootNeeded = $true
        }}
    }} catch {{
        $failedCount++
        $lastError = $_.Exception.Message
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

$reboot = $rebootNeeded -or ({REBOOT_PENDING_EXPR})
$succeeded = ($failedCount -eq 0 -and $succeededCount -gt 0)
$msg = if ($succeeded) {{
    "Installation completed successfully."
}} elseif ($succeededCount -gt 0) {{
    if ($lastError) {{
        "Installation completed with errors ($succeededCount succeeded, $failedCount failed): $lastError"
    }} else {{
        "Installation completed with errors ($succeededCount succeeded, $failedCount failed)."
    }}
}} else {{
    if ($lastError) {{
        "Installation failed: $lastError"
    }} else {{
        "Installation failed. Administrative privileges or a system restart may be required."
    }}
}}

[PSCustomObject]@{{
    Success = $succeeded
    Message = $msg
    SucceededCount = $succeededCount
    FailedCount = $failedCount
    RebootRequired = $reboot
}} | ConvertTo-Json -Compress
"#
    )
}
