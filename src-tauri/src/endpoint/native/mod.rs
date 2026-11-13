#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(any(target_os = "linux", test))]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(any(target_os = "macos", test))]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod fallback {
    use super::super::types::{BatteryInfo, GpuInfo, MotherboardInfo};

    pub fn query_battery_status() -> Result<Option<BatteryInfo>, String> {
        Ok(None)
    }

    pub fn query_domain_or_workgroup() -> Result<String, String> {
        Ok("WORKGROUP".to_string())
    }

    pub fn query_build_number() -> Result<String, String> {
        Ok(String::new())
    }

    pub fn query_motherboard() -> Result<Option<MotherboardInfo>, String> {
        Ok(None)
    }

    pub fn query_gpus() -> Result<Vec<GpuInfo>, String> {
        Ok(Vec::new())
    }

    pub fn query_fast_startup_enabled() -> Result<bool, String> {
        Ok(false)
    }

    pub fn query_is_admin() -> Result<bool, String> {
        Ok(false)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub use fallback::*;

#[cfg(any(target_os = "linux", target_os = "macos", test))]
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
pub(super) fn run_command_stdout(program: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute {program}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("{program} exited with status {:?}", output.status.code())
        } else {
            format!("{program} failed: {stderr}")
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
pub(super) fn query_unix_is_admin() -> Result<bool, String> {
    let uid = run_command_stdout("id", &["-u"])?;
    Ok(uid.trim() == "0")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn make_temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "caffeine_native_test_{}_{}_{}",
            label,
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn test_query_battery_status_runs_without_panicking() {
        let result = query_battery_status();
        assert!(
            result.is_ok(),
            "query_battery_status returned an error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_linux_parse_battery_sysfs() {
        let charging = linux::parse_battery_sysfs("85\n", "Charging\n").unwrap();
        assert!(charging.has_battery);
        assert!(charging.is_charging);
        assert_eq!(charging.charge_percent, 85);

        let full = linux::parse_battery_sysfs("100", "Full").unwrap();
        assert!(full.is_charging);
        assert_eq!(full.charge_percent, 100);

        let discharging = linux::parse_battery_sysfs("42", "Discharging").unwrap();
        assert!(!discharging.is_charging);
        assert_eq!(discharging.charge_percent, 42);

        assert!(linux::parse_battery_sysfs("invalid", "Charging").is_err());
    }

    #[test]
    fn test_linux_query_battery_status_from_dir() {
        let missing = PathBuf::from("/nonexistent_caffeine_power_supply_dir");
        assert!(linux::query_battery_status_from_dir(&missing)
            .unwrap()
            .is_none());

        let root = make_temp_dir("linux_bat");
        let mouse_bat = root.join("hidpp_battery_0");
        fs::create_dir_all(&mouse_bat).unwrap();
        fs::write(mouse_bat.join("type"), "Battery\n").unwrap();
        fs::write(mouse_bat.join("scope"), "Device\n").unwrap();
        fs::write(mouse_bat.join("capacity"), "90\n").unwrap();
        fs::write(mouse_bat.join("status"), "Discharging\n").unwrap();

        let ac = root.join("AC");
        fs::create_dir_all(&ac).unwrap();
        fs::write(ac.join("type"), "Mains\n").unwrap();

        let bat0 = root.join("BAT0");
        fs::create_dir_all(&bat0).unwrap();
        fs::write(bat0.join("type"), "Battery\n").unwrap();
        fs::write(bat0.join("capacity"), "63\n").unwrap();
        fs::write(bat0.join("status"), "Charging\n").unwrap();

        let info = linux::query_battery_status_from_dir(&root)
            .unwrap()
            .expect("should detect BAT0 and skip Device-scoped mouse battery");
        assert!(info.is_charging);
        assert_eq!(info.charge_percent, 63);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_linux_parse_resolv_conf_domain() {
        let with_domain = "# comment\nsearch fallback.local\ndomain corp.example.com\nnameserver 1.1.1.1\n";
        assert_eq!(
            linux::parse_resolv_conf_domain(with_domain).as_deref(),
            Some("corp.example.com")
        );

        let with_search = "search lab.internal corp.local\nnameserver 8.8.8.8\n";
        assert_eq!(
            linux::parse_resolv_conf_domain(with_search).as_deref(),
            Some("lab.internal")
        );

        let empty = "# only comments\nnameserver 127.0.0.53\n";
        assert_eq!(linux::parse_resolv_conf_domain(empty), None);
    }

    #[test]
    fn test_linux_query_motherboard_from_dir() {
        let missing = PathBuf::from("/nonexistent_caffeine_dmi_dir");
        assert!(linux::query_motherboard_from_dir(&missing)
            .unwrap()
            .is_none());

        let dir = make_temp_dir("linux_dmi");
        fs::write(dir.join("sys_vendor"), "LENOVO\n").unwrap();
        fs::write(dir.join("product_name"), "ThinkPad T14\n").unwrap();
        fs::write(dir.join("bios_vendor"), "LENOVO\n").unwrap();
        fs::write(dir.join("bios_version"), "N2XET38W\n").unwrap();
        fs::write(dir.join("bios_date"), "01/15/2024\n").unwrap();

        let info = linux::query_motherboard_from_dir(&dir)
            .unwrap()
            .expect("should build motherboard from fallback sys_vendor/product_name");
        assert_eq!(info.manufacturer, "LENOVO");
        assert_eq!(info.product, "ThinkPad T14");
        assert_eq!(info.bios_version, "N2XET38W");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_linux_drm_and_lspci_parsing() {
        assert!(linux::is_primary_drm_card("card0"));
        assert!(linux::is_primary_drm_card("card12"));
        assert!(!linux::is_primary_drm_card("card0-DP-1"));
        assert!(!linux::is_primary_drm_card("renderD128"));
        assert!(!linux::is_primary_drm_card("card"));

        let uevent = "DRIVER=amdgpu\nPCI_CLASS=30000\nPCI_SLOT_NAME=0000:03:00.0\n";
        let (driver, slot) = linux::parse_drm_uevent(uevent);
        assert_eq!(driver.as_deref(), Some("amdgpu"));
        assert_eq!(slot.as_deref(), Some("0000:03:00.0"));

        let lspci = "0000:00:02.0 Host bridge: Intel Corp Device\n\
                     0000:03:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Navi 23\n";
        let map = linux::parse_lspci_gpu_map(lspci);
        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("0000:03:00.0").map(String::as_str),
            Some("Advanced Micro Devices, Inc. [AMD/ATI] Navi 23")
        );

        let drm = make_temp_dir("linux_drm");
        let card0_dev = drm.join("card0").join("device");
        let conn = drm.join("card0-DP-1");
        fs::create_dir_all(&card0_dev).unwrap();
        fs::create_dir_all(&conn).unwrap();
        fs::write(card0_dev.join("uevent"), uevent).unwrap();
        fs::write(card0_dev.join("mem_info_vram_total"), "8589934592\n").unwrap();

        let gpus = linux::query_gpus_from_sysfs(&drm, &map).unwrap();
        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].name, "Advanced Micro Devices, Inc. [AMD/ATI] Navi 23");
        assert_eq!(gpus[0].driver_version, "amdgpu");
        assert_eq!(gpus[0].vram_bytes, 8_589_934_592);

        let _ = fs::remove_dir_all(&drm);
    }

    #[test]
    fn test_macos_parse_pmset_batt() {
        let charging = "Now drawing from 'AC Power'\n -InternalBattery-0 (id=123)\t85%; charging; 0:45 remaining present: true\n";
        let b1 = macos::parse_pmset_batt(charging).unwrap().unwrap();
        assert_eq!(b1.charge_percent, 85);
        assert!(b1.is_charging);

        let discharging = "Now drawing from 'Battery Power'\n -InternalBattery-0 (id=123)\t72%; discharging; 4:12 remaining present: true\n";
        let b2 = macos::parse_pmset_batt(discharging).unwrap().unwrap();
        assert_eq!(b2.charge_percent, 72);
        assert!(!b2.is_charging);

        let desktop = "Now drawing from 'AC Power'\n";
        assert!(macos::parse_pmset_batt(desktop).unwrap().is_none());

        let malformed = "Now drawing from 'AC Power'\n -InternalBattery-0 (id=123)\tno_percent; charging\n";
        assert!(macos::parse_pmset_batt(malformed).is_err());
    }

    #[test]
    fn test_macos_domain_parsing() {
        let scutil = "DNS configuration\n\nresolver #1\n  search domain[0] : corp.apple.internal\n  nameserver[0] : 10.0.0.1\n";
        assert_eq!(
            macos::parse_scutil_dns_domain(scutil).as_deref(),
            Some("corp.apple.internal")
        );
        assert_eq!(
            macos::parse_hostname_domain("macbook-pro.office.local\n").as_deref(),
            Some("office.local")
        );
        assert_eq!(macos::parse_hostname_domain("macbook-pro"), None);
    }

    #[test]
    fn test_macos_parse_hardware_json() {
        let json = r#"{
            "SPHardwareDataType": [
                {
                    "_name": "hardware_overview",
                    "machine_model": "Mac14,9",
                    "model_identifier": "Mac14,9",
                    "boot_rom_version": "10151.101.3"
                }
            ]
        }"#;
        let mb = macos::parse_macos_hardware_json(json).unwrap().unwrap();
        assert_eq!(mb.manufacturer, "Apple Inc.");
        assert_eq!(mb.product, "Mac14,9");
        assert_eq!(mb.bios_version, "10151.101.3");

        let empty = r#"{"SPHardwareDataType": []}"#;
        assert!(macos::parse_macos_hardware_json(empty).unwrap().is_none());
        assert!(macos::parse_macos_hardware_json("not json").is_err());
    }

    #[test]
    fn test_macos_parse_displays_json() {
        let json = r#"{
            "SPDisplaysDataType": [
                {
                    "_name": "Apple M2 Pro",
                    "sppci_model": "Apple M2 Pro",
                    "spdisplays_mtlgpufamilysupport": "spdisplays_metal3",
                    "spdisplays_vram": "16 GB"
                },
                {
                    "_name": "Intel UHD Graphics 630",
                    "sppci_model": "Intel UHD Graphics 630",
                    "spdisplays_metal": "Supported, feature set macOS GPUFamily2 v1",
                    "spdisplays_vram_shared": "1536 MB"
                }
            ]
        }"#;
        let gpus = macos::parse_macos_displays_json(json).unwrap();
        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].name, "Apple M2 Pro");
        assert_eq!(gpus[0].driver_version, "metal3");
        assert_eq!(gpus[0].vram_bytes, 16 * 1024 * 1024 * 1024);

        assert_eq!(gpus[1].name, "Intel UHD Graphics 630");
        assert_eq!(gpus[1].vram_bytes, 1536 * 1024 * 1024);
    }
}
