#[cfg(any(target_os = "linux", test))]
use super::super::types::SystemUpdateItem;
#[cfg(target_os = "linux")]
use super::super::types::{UpdateCheckResult, UpdateOperationResult, UpdateProgressEvent};
#[cfg(target_os = "linux")]
use super::types::{
    current_utc_timestamp, format_command_failure, resolve_target_ids, run_package_batch,
};

#[cfg(any(target_os = "linux", test))]
fn build_package_item(
    id: String,
    title: String,
    description: Option<String>,
    categories: Vec<String>,
    severity: Option<String>,
) -> SystemUpdateItem {
    SystemUpdateItem {
        id,
        title,
        kb_article_id: None,
        description,
        size_bytes: None,
        categories,
        severity,
        is_mandatory: false,
        is_downloaded: false,
        support_url: None,
    }
}

#[cfg(any(target_os = "linux", test))]
pub(crate) fn parse_apt_upgradable(stdout: &str) -> Vec<SystemUpdateItem> {
    let mut items = Vec::new();
    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("Listing...") {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(pkg_token) = parts.next() else {
            continue;
        };
        let Some((name, repo)) = pkg_token.split_once('/') else {
            continue;
        };
        if name.is_empty() {
            continue;
        }

        let new_version = parts.next();
        let arch = parts.next();
        let old_version = line
            .find("[upgradable from:")
            .and_then(|idx| line[idx + "[upgradable from:".len()..].split(']').next())
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let title = match new_version {
            Some(ver) => format!("{name} ({ver})"),
            None => name.to_string(),
        };

        let mut desc_parts = Vec::new();
        match (old_version, new_version) {
            (Some(old), Some(new)) => desc_parts.push(format!("Upgrade from {old} to {new}")),
            (None, Some(new)) => desc_parts.push(format!("Available version: {new}")),
            _ => {}
        }
        if let Some(a) = arch {
            desc_parts.push(format!("[{a}]"));
        }
        let description = (!desc_parts.is_empty()).then(|| desc_parts.join(" "));

        let is_security = repo.to_ascii_lowercase().contains("security");
        let categories = if is_security {
            vec!["Security".to_string()]
        } else {
            vec!["Package Update".to_string()]
        };
        let severity = is_security.then(|| "Important".to_string());

        items.push(build_package_item(
            name.to_string(),
            title,
            description,
            categories,
            severity,
        ));
    }
    items
}

#[cfg(any(target_os = "linux", test))]
pub(crate) fn parse_dnf_check_update(stdout: &str) -> Vec<SystemUpdateItem> {
    let mut items = Vec::new();
    for raw_line in stdout.lines() {
        if raw_line.starts_with(char::is_whitespace) {
            continue;
        }
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with("Last ")
            || line.starts_with("Security:")
            || line.eq_ignore_ascii_case("Obsoleting Packages")
        {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 || parts[2].starts_with('@') {
            continue;
        }

        let name = parts[0];
        let version = parts[1];
        let repo = parts[2];

        let is_security = repo.to_ascii_lowercase().contains("security");
        items.push(build_package_item(
            name.to_string(),
            format!("{name} ({version})"),
            Some(format!("Repository: {repo}, Version: {version}")),
            vec![repo.to_string()],
            is_security.then(|| "Important".to_string()),
        ));
    }
    items
}

#[cfg(any(target_os = "linux", test))]
pub(crate) fn parse_pacman_qu(stdout: &str) -> Vec<SystemUpdateItem> {
    let mut items = Vec::new();
    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 || parts[2] != "->" {
            continue;
        }
        if parts.get(4).is_some_and(|flag| *flag == "[ignored]") {
            continue;
        }

        let name = parts[0];
        let old_ver = parts[1];
        let new_ver = parts[3];

        items.push(build_package_item(
            name.to_string(),
            format!("{name} ({new_ver})"),
            Some(format!("Upgrade from {old_ver} to {new_ver}")),
            vec!["Package Update".to_string()],
            None,
        ));
    }
    items
}

#[cfg(any(target_os = "linux", test))]
pub(crate) fn parse_zypper_updates(stdout: &str) -> Vec<SystemUpdateItem> {
    let mut items = Vec::new();
    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        if !line.contains('|') {
            continue;
        }

        let cols: Vec<&str> = line.split('|').map(str::trim).collect();
        if cols.len() < 6 {
            continue;
        }
        if cols[0].is_empty()
            || cols[0] == "S"
            || cols[0].starts_with('-')
            || cols[2].is_empty()
            || cols[2] == "Name"
        {
            continue;
        }

        let repo = cols[1];
        let name = cols[2];
        let current_ver = cols[3];
        let avail_ver = cols[4];
        let arch = cols[5];

        let categories = if repo.is_empty() {
            vec!["Package Update".to_string()]
        } else {
            vec![repo.to_string()]
        };

        items.push(build_package_item(
            name.to_string(),
            format!("{name} ({avail_ver})"),
            Some(format!("Upgrade from {current_ver} to {avail_ver} [{arch}]")),
            categories,
            None,
        ));
    }
    items
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Zypper,
}

#[cfg(target_os = "linux")]
fn detect_package_manager() -> Result<PackageManager, String> {
    if crate::process::find_executable_in_path("apt").is_some() {
        Ok(PackageManager::Apt)
    } else if crate::process::find_executable_in_path("dnf").is_some() {
        Ok(PackageManager::Dnf)
    } else if crate::process::find_executable_in_path("pacman").is_some() {
        Ok(PackageManager::Pacman)
    } else if crate::process::find_executable_in_path("zypper").is_some() {
        Ok(PackageManager::Zypper)
    } else {
        Err("No supported Linux package manager found in PATH (apt, dnf, pacman, zypper).".to_string())
    }
}

#[cfg(target_os = "linux")]
fn is_reboot_required() -> bool {
    if std::path::Path::new("/var/run/reboot-required").exists() {
        return true;
    }
    if crate::process::find_executable_in_path("needs-restarting").is_some() {
        if let Ok(status) = std::process::Command::new("needs-restarting")
            .arg("-r")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            // needs-restarting -r exits with 1 specifically when a reboot is required.
            return status.code() == Some(1);
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn query_upgradable_items(pm: PackageManager) -> Result<Vec<SystemUpdateItem>, String> {
    match pm {
        PackageManager::Apt => {
            let output = std::process::Command::new("apt")
                .args(["list", "--upgradable"])
                .env("DEBIAN_FRONTEND", "noninteractive")
                .output()
                .map_err(|e| format!("Failed to execute apt list --upgradable: {e}"))?;
            if !output.status.success() {
                return Err(format_command_failure("apt list --upgradable", &output));
            }
            Ok(parse_apt_upgradable(&String::from_utf8_lossy(&output.stdout)))
        }
        PackageManager::Dnf => {
            let output = std::process::Command::new("dnf")
                .arg("check-update")
                .output()
                .map_err(|e| format!("Failed to execute dnf check-update: {e}"))?;
            // dnf check-update exits 100 when updates are available and 0 when none are available.
            match output.status.code() {
                Some(0) => Ok(Vec::new()),
                Some(100) => Ok(parse_dnf_check_update(&String::from_utf8_lossy(&output.stdout))),
                _ => Err(format_command_failure("dnf check-update", &output)),
            }
        }
        PackageManager::Pacman => {
            let output = std::process::Command::new("pacman")
                .arg("-Qu")
                .output()
                .map_err(|e| format!("Failed to execute pacman -Qu: {e}"))?;
            if output.status.success() {
                return Ok(parse_pacman_qu(&String::from_utf8_lossy(&output.stdout)));
            }
            // pacman -Qu exits 1 with empty output when no updates are available.
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.code() == Some(1) && output.stdout.is_empty() && stderr.trim().is_empty()
            {
                return Ok(Vec::new());
            }
            Err(format_command_failure("pacman -Qu", &output))
        }
        PackageManager::Zypper => {
            let output = std::process::Command::new("zypper")
                .arg("list-updates")
                .output()
                .map_err(|e| format!("Failed to execute zypper list-updates: {e}"))?;
            if !output.status.success() {
                return Err(format_command_failure("zypper list-updates", &output));
            }
            Ok(parse_zypper_updates(&String::from_utf8_lossy(&output.stdout)))
        }
    }
}

#[cfg(target_os = "linux")]
fn run_package_command(pm: PackageManager, pkg: &str, download_only: bool) -> Result<(), String> {
    let mut cmd = match (pm, download_only) {
        (PackageManager::Apt, true) => {
            let mut c = std::process::Command::new("apt");
            c.args(["install", "-y", "--download-only", pkg]);
            c.env("DEBIAN_FRONTEND", "noninteractive");
            c
        }
        (PackageManager::Apt, false) => {
            let mut c = std::process::Command::new("apt");
            c.args(["install", "-y", pkg]);
            c.env("DEBIAN_FRONTEND", "noninteractive");
            c
        }
        (PackageManager::Dnf, true) => {
            let mut c = std::process::Command::new("dnf");
            c.args(["upgrade", "--downloadonly", "-y", pkg]);
            c
        }
        (PackageManager::Dnf, false) => {
            let mut c = std::process::Command::new("dnf");
            c.args(["upgrade", "-y", pkg]);
            c
        }
        (PackageManager::Pacman, true) => {
            let mut c = std::process::Command::new("pacman");
            c.args(["-Sw", "--noconfirm", pkg]);
            c
        }
        (PackageManager::Pacman, false) => {
            let mut c = std::process::Command::new("pacman");
            c.args(["-S", "--noconfirm", pkg]);
            c
        }
        (PackageManager::Zypper, true) => {
            let mut c = std::process::Command::new("zypper");
            c.args(["--non-interactive", "install", "--download-only", pkg]);
            c
        }
        (PackageManager::Zypper, false) => {
            let mut c = std::process::Command::new("zypper");
            c.args(["--non-interactive", "update", pkg]);
            c
        }
    };

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute package manager for '{pkg}': {e}"))?;
    if !output.status.success() {
        let op_name = if download_only { "download" } else { "install" };
        return Err(format_command_failure(
            &format!("{op_name} '{pkg}'"),
            &output,
        ));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub(crate) fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let pm = detect_package_manager()?;
    let updates = query_upgradable_items(pm)?;
    let last_checked = current_utc_timestamp()?;

    Ok(UpdateCheckResult {
        updates,
        reboot_required: is_reboot_required(),
        last_checked,
    })
}

#[cfg(target_os = "linux")]
fn execute_linux_update_operation<F>(
    phase: &str,
    download_only: bool,
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    let pm = detect_package_manager()?;
    let mut targets = resolve_target_ids(target_ids, target_id)?;
    if targets.is_empty() {
        targets = query_upgradable_items(pm)?
            .into_iter()
            .map(|u| u.id)
            .collect();
    }

    let empty_message = if download_only {
        "All selected updates are already downloaded."
    } else {
        "No updates available to install."
    };

    Ok(run_package_batch(
        phase,
        targets,
        empty_message,
        on_progress,
        |pkg| run_package_command(pm, pkg, download_only),
        is_reboot_required,
    ))
}

#[cfg(target_os = "linux")]
pub(crate) fn download_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    execute_linux_update_operation("download", true, target_ids, target_id, on_progress)
}

#[cfg(target_os = "linux")]
pub(crate) fn install_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    execute_linux_update_operation("install", false, target_ids, target_id, on_progress)
}

#[cfg(target_os = "linux")]
pub(crate) fn restart_system() -> Result<(), String> {
    let systemctl_err = match std::process::Command::new("systemctl").arg("reboot").output() {
        Ok(out) if out.status.success() => return Ok(()),
        Ok(out) => format_command_failure("systemctl reboot", &out),
        Err(e) => format!("Failed to execute systemctl reboot: {e}"),
    };

    let shutdown_out = std::process::Command::new("shutdown")
        .args(["-r", "now"])
        .output()
        .map_err(|e| {
            format!("System restart failed ({systemctl_err}); failed to execute shutdown: {e}")
        })?;

    if !shutdown_out.status.success() {
        return Err(format!(
            "System restart failed ({systemctl_err}); {}",
            format_command_failure("shutdown -r now", &shutdown_out)
        ));
    }

    Ok(())
}
