mod linux;
mod macos;
pub mod scripts;
pub mod types;
#[cfg(target_os = "windows")]
mod windows;

use super::types::{UpdateCheckResult, UpdateOperationResult, UpdateProgressEvent};
pub use types::{format_powershell_id_array, resolve_target_ids};

pub fn check_for_updates() -> Result<UpdateCheckResult, String> {
    #[cfg(target_os = "windows")]
    {
        windows::check_for_updates()
    }
    #[cfg(target_os = "linux")]
    {
        linux::check_for_updates()
    }
    #[cfg(target_os = "macos")]
    {
        macos::check_for_updates()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("System updates are not supported on this operating system.".to_string())
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
    #[cfg(target_os = "windows")]
    {
        windows::download_updates(target_ids, target_id, on_progress)
    }
    #[cfg(target_os = "linux")]
    {
        linux::download_updates(target_ids, target_id, on_progress)
    }
    #[cfg(target_os = "macos")]
    {
        macos::download_updates(target_ids, target_id, on_progress)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = (target_ids, target_id, on_progress);
        Err("System updates are not supported on this operating system.".to_string())
    }
}

pub fn install_updates<F>(
    target_ids: Option<Vec<String>>,
    target_id: Option<String>,
    on_progress: F,
) -> Result<UpdateOperationResult, String>
where
    F: FnMut(UpdateProgressEvent),
{
    #[cfg(target_os = "windows")]
    {
        windows::install_updates(target_ids, target_id, on_progress)
    }
    #[cfg(target_os = "linux")]
    {
        linux::install_updates(target_ids, target_id, on_progress)
    }
    #[cfg(target_os = "macos")]
    {
        macos::install_updates(target_ids, target_id, on_progress)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = (target_ids, target_id, on_progress);
        Err("System updates are not supported on this operating system.".to_string())
    }
}

pub fn restart_system() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        windows::restart_system()
    }
    #[cfg(target_os = "linux")]
    {
        linux::restart_system()
    }
    #[cfg(target_os = "macos")]
    {
        macos::restart_system()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("System updates are not supported on this operating system.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::types::{
        current_utc_timestamp, run_package_batch, RawProgress, RawUpdateOperation,
    };
    use super::*;

    #[test]
    fn test_resolve_target_ids_merges_and_sanitizes() {
        let list = Some(vec!["update-1".to_string(), "update-2".to_string()]);
        let single = Some("update-3".to_string());
        let resolved = resolve_target_ids(list, single).unwrap();
        expect_eq(resolved, vec!["update-1", "update-2", "update-3"]);
    }

    #[test]
    fn test_resolve_target_ids_allows_cross_platform_identifiers() {
        let ids = Some(vec![
            "{12345678-1234-1234-1234-1234567890ab}".to_string(),
            "curl.x86_64".to_string(),
            "libstdc++6".to_string(),
            "openssl-libs:1.0~rc1".to_string(),
            "python@3.12_1".to_string(),
        ]);
        let resolved = resolve_target_ids(ids, None).unwrap();
        assert_eq!(resolved.len(), 5);
    }

    #[test]
    fn test_resolve_target_ids_rejects_special_characters() {
        let invalid = [
            "update; Remove-Item C:\\",
            "pkg with spaces",
            "pkg/sub",
            "pkg\\sub",
            "pkg'quote",
            "pkg\"quote",
            "pkg|cat",
            "pkg$HOME",
            "pkg`id`",
            "",
        ];
        for bad in invalid {
            assert!(
                resolve_target_ids(Some(vec![bad.to_string()]), None).is_err(),
                "expected '{bad}' to be rejected"
            );
        }
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

    #[test]
    fn test_parse_apt_upgradable() {
        let sample = "\
Listing... Done
curl/jammy-updates,jammy-security 7.81.0-1ubuntu1.16 amd64 [upgradable from: 7.81.0-1ubuntu1.15]
tzdata/jammy-updates 2024a-0ubuntu0.22.04 all [upgradable from: 2023d-0ubuntu0.22.04]
";
        let items = linux::parse_apt_upgradable(sample);
        assert_eq!(items.len(), 2);

        assert_eq!(items[0].id, "curl");
        assert_eq!(items[0].title, "curl (7.81.0-1ubuntu1.16)");
        assert_eq!(
            items[0].description.as_deref(),
            Some("Upgrade from 7.81.0-1ubuntu1.15 to 7.81.0-1ubuntu1.16 [amd64]")
        );
        assert_eq!(items[0].categories, vec!["Security"]);
        assert_eq!(items[0].severity.as_deref(), Some("Important"));

        assert_eq!(items[1].id, "tzdata");
        assert_eq!(items[1].categories, vec!["Package Update"]);
        assert_eq!(items[1].severity, None);
    }

    #[test]
    fn test_parse_apt_upgradable_empty() {
        assert!(linux::parse_apt_upgradable("Listing... Done\n").is_empty());
        assert!(linux::parse_apt_upgradable("").is_empty());
    }

    #[test]
    fn test_parse_dnf_check_update() {
        let sample = "\
Last metadata expiration check: 0:12:34 ago on Mon 13 Nov 2026 10:00:00 AM UTC.

curl.x86_64                    8.5.0-2.fc40             updates
openssl-libs.x86_64            1:3.2.1-2.fc40           updates-security
Obsoleting Packages
foo.x86_64                     1.0-1.fc40               updates
    bar.x86_64                 0.9-1.fc40               @System
Security: kernel-core-6.8.9-300.fc40.x86_64 is an installed security update
";
        let items = linux::parse_dnf_check_update(sample);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].id, "curl.x86_64");
        assert_eq!(items[0].title, "curl.x86_64 (8.5.0-2.fc40)");
        assert_eq!(items[0].categories, vec!["updates"]);
        assert_eq!(items[0].severity, None);

        assert_eq!(items[1].id, "openssl-libs.x86_64");
        assert_eq!(items[1].severity.as_deref(), Some("Important"));
        assert_eq!(items[2].id, "foo.x86_64");
    }

    #[test]
    fn test_parse_pacman_qu() {
        let sample = "\
curl 8.7.1-1 -> 8.8.0-1
linux 6.8.9.arch1-1 -> 6.9.1.arch1-1
openssl 3.2.1-1 -> 3.3.0-1 [ignored]
";
        let items = linux::parse_pacman_qu(sample);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "curl");
        assert_eq!(items[0].title, "curl (8.8.0-1)");
        assert_eq!(
            items[0].description.as_deref(),
            Some("Upgrade from 8.7.1-1 to 8.8.0-1")
        );
        assert_eq!(items[1].id, "linux");
    }

    #[test]
    fn test_parse_zypper_updates() {
        let sample = "\
Loading repository data...
Reading installed packages...
S | Repository              | Name      | Current Version | Available Version | Arch
--+-------------------------+-----------+-----------------+-------------------+-------
v | Main Update Repository  | curl      | 8.0.1-1.1       | 8.6.0-1.1         | x86_64
v | Update repository (SLE) | openssl-3 | 3.1.4-1.1       | 3.1.4-2.1         | x86_64
";
        let items = linux::parse_zypper_updates(sample);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "curl");
        assert_eq!(items[0].title, "curl (8.6.0-1.1)");
        assert_eq!(items[0].categories, vec!["Main Update Repository"]);
        assert_eq!(
            items[0].description.as_deref(),
            Some("Upgrade from 8.0.1-1.1 to 8.6.0-1.1 [x86_64]")
        );
        assert_eq!(items[1].id, "openssl-3");
        assert_eq!(items[1].categories, vec!["Update repository (SLE)"]);
    }

    #[test]
    fn test_parse_zypper_updates_no_updates() {
        let sample = "Loading repository data...\nReading installed packages...\nNo updates found.\n";
        assert!(linux::parse_zypper_updates(sample).is_empty());
    }

    #[test]
    fn test_parse_softwareupdate_list() {
        let sample = "\
Software Update Tool

Finding available software
Software Update found the following new or updated software:
* Label: macOS Sonoma 14.5-23F79
\tTitle: macOS Sonoma 14.5, Version: 14.5, Size: 1384243KiB, Recommended: YES, Action: restart, 
* Label: Safari17.5SonomaAuto-17.5
\tTitle: Safari, Version: 17.5, Size: 175616K, Recommended: NO, 
";
        let (items, reboot_required) = macos::parse_softwareupdate_list(sample);
        assert!(reboot_required);
        assert_eq!(items.len(), 2);

        assert_eq!(items[0].id, "macOS Sonoma 14.5-23F79");
        assert_eq!(items[0].title, "macOS Sonoma 14.5");
        assert_eq!(items[0].size_bytes, Some(1_384_243 * 1024));
        assert!(items[0].is_mandatory);
        assert_eq!(items[0].categories, vec!["Recommended"]);
        assert_eq!(items[0].severity.as_deref(), Some("Important"));
        assert_eq!(
            items[0].description.as_deref(),
            Some("Version: 14.5, Action: restart")
        );

        assert_eq!(items[1].id, "Safari17.5SonomaAuto-17.5");
        assert_eq!(items[1].title, "Safari");
        assert_eq!(items[1].size_bytes, Some(175_616 * 1024));
        assert!(!items[1].is_mandatory);
        assert_eq!(items[1].categories, vec!["Software Update"]);
        assert_eq!(items[1].severity, None);
    }

    #[test]
    fn test_parse_softwareupdate_list_no_updates() {
        let sample = "Software Update Tool\n\nFinding available software\nNo new software available.\n";
        let (items, reboot_required) = macos::parse_softwareupdate_list(sample);
        assert!(items.is_empty());
        assert!(!reboot_required);
    }

    #[test]
    fn test_run_package_batch_emits_progress_and_handles_partial_failure() {
        let mut events = Vec::new();
        let res = run_package_batch(
            "install",
            vec!["curl".to_string(), "badpkg".to_string()],
            "none",
            |ev| events.push(ev),
            |pkg| {
                if pkg == "badpkg" {
                    Err("exit code 1".to_string())
                } else {
                    Ok(())
                }
            },
            || true,
        );

        assert!(!res.success);
        assert!(res.reboot_required);
        assert_eq!(res.succeeded_count, 1);
        assert_eq!(res.failed_count, 1);
        assert!(res.message.contains("exit code 1"));
        assert_eq!(events.len(), 4);
        assert_eq!(events[0].percent_complete, 0);
        assert_eq!(events[1].percent_complete, 50);
        assert_eq!(events[2].percent_complete, 50);
        assert_eq!(events[3].percent_complete, 100);
    }

    #[test]
    fn test_current_utc_timestamp_format() {
        let ts = current_utc_timestamp().unwrap();
        assert!(ts.ends_with(" UTC"), "unexpected timestamp: {ts}");
        assert_eq!(ts.len(), 23);
    }

    fn expect_eq(actual: Vec<String>, expected: Vec<&str>) {
        let expected_strings: Vec<String> = expected.into_iter().map(String::from).collect();
        assert_eq!(actual, expected_strings);
    }
}
