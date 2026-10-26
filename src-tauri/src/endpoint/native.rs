#[cfg(windows)]
mod win {
    use super::super::types::{BatteryInfo, GpuInfo, MotherboardInfo};
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::NetworkManagement::NetManagement::{
        NetApiBufferFree, NetGetJoinInformation,
    };
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    pub fn query_battery_status() -> Result<Option<BatteryInfo>, String> {
        let mut status = SYSTEM_POWER_STATUS {
            ACLineStatus: 0,
            BatteryFlag: 0,
            BatteryLifePercent: 0,
            SystemStatusFlag: 0,
            BatteryLifeTime: 0,
            BatteryFullLifeTime: 0,
        };

        let success = unsafe { GetSystemPowerStatus(&mut status) };
        if success == 0 {
            return Err("GetSystemPowerStatus Win32 call failed".to_string());
        }

        if status.BatteryFlag == 128
            || (status.BatteryFlag == 255 && status.BatteryLifePercent == 255)
        {
            return Ok(None);
        }

        let is_charging = (status.BatteryFlag & 8) != 0 || status.ACLineStatus == 1;
        let charge_percent = if status.BatteryLifePercent <= 100 {
            status.BatteryLifePercent
        } else {
            0
        };

        Ok(Some(BatteryInfo {
            has_battery: true,
            is_charging,
            charge_percent,
        }))
    }

    pub fn query_domain_or_workgroup() -> Result<String, String> {
        let mut name_buf: *mut u16 = null_mut();
        let mut join_status = 0;

        let status = unsafe { NetGetJoinInformation(null_mut(), &mut name_buf, &mut join_status) };
        if status != 0 {
            return Err(format!("NetGetJoinInformation failed with code {status}"));
        }

        if name_buf.is_null() {
            return Ok("WORKGROUP".to_string());
        }

        let name = unsafe {
            let mut len = 0;
            while *name_buf.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(name_buf, len);
            let s = String::from_utf16_lossy(slice);
            NetApiBufferFree(name_buf.cast());
            s
        };

        if name.trim().is_empty() {
            Ok("WORKGROUP".to_string())
        } else {
            Ok(name.trim().to_string())
        }
    }

    pub fn query_build_number() -> Result<String, String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hklm
            .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
            .map_err(|e| format!("Failed to open Windows NT CurrentVersion registry key: {e}"))?;

        key.get_value::<String, _>("CurrentBuildNumber")
            .or_else(|_| key.get_value::<String, _>("CurrentBuild"))
            .map_err(|e| format!("Failed to read CurrentBuildNumber from registry: {e}"))
    }

    pub fn query_motherboard() -> Result<Option<MotherboardInfo>, String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let bios_key = hklm
            .open_subkey("HARDWARE\\DESCRIPTION\\System\\BIOS")
            .map_err(|e| format!("Failed to open BIOS registry key: {e}"))?;

        let manufacturer: String = bios_key
            .get_value("BaseBoardManufacturer")
            .or_else(|_| bios_key.get_value("SystemManufacturer"))
            .unwrap_or_default();

        let product: String = bios_key
            .get_value("BaseBoardProduct")
            .or_else(|_| bios_key.get_value("SystemProductName"))
            .unwrap_or_default();

        let bios_vendor: String = bios_key.get_value("BIOSVendor").unwrap_or_default();
        let bios_version: String = bios_key.get_value("BIOSVersion").unwrap_or_default();
        let bios_date: String = bios_key.get_value("BIOSReleaseDate").unwrap_or_default();

        Ok(Some(MotherboardInfo {
            manufacturer,
            product,
            bios_vendor,
            bios_version,
            bios_date,
        }))
    }

    pub fn query_gpus() -> Result<Vec<GpuInfo>, String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let display_class = hklm
            .open_subkey(
                "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}",
            )
            .map_err(|e| format!("Failed to open display adapter class registry key: {e}"))?;

        let mut gpus = Vec::new();
        for subkey_name in display_class.enum_keys().filter_map(|r| r.ok()) {
            if subkey_name.len() != 4 || !subkey_name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            let adapter_key = match display_class.open_subkey(&subkey_name) {
                Ok(k) => k,
                Err(_) => continue,
            };

            let name: String = match adapter_key.get_value("DriverDesc") {
                Ok(desc) => desc,
                Err(_) => continue,
            };

            let lower_name = name.to_lowercase();
            if lower_name.contains("remote display") || lower_name.contains("indirect display") {
                continue;
            }

            let driver_version: String = adapter_key.get_value("DriverVersion").unwrap_or_default();
            let vram_bytes: u64 = adapter_key
                .get_value::<u64, _>("HardwareInformation.qwMemorySize")
                .or_else(|_| {
                    adapter_key
                        .get_value::<u32, _>("HardwareInformation.MemorySize")
                        .map(|v| v as u64)
                })
                .unwrap_or(0);

            gpus.push(GpuInfo {
                name,
                driver_version,
                vram_bytes,
                temperature_celsius: None,
            });
        }

        Ok(gpus)
    }

    pub fn query_fast_startup_enabled() -> Result<bool, String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = match hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Power") {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(format!("Failed to open Power registry key: {e}")),
        };

        match key.get_value::<u32, _>("HiberbootEnabled") {
            Ok(val) => Ok(val == 1),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(format!("Failed to read HiberbootEnabled registry value: {e}")),
        }
    }

    pub fn query_is_admin() -> Result<bool, String> {
        let mut token = null_mut();
        let success = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) };
        if success == 0 {
            return Err("Failed to open process token".to_string());
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0;
        let success = unsafe {
            GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            )
        };

        unsafe { CloseHandle(token) };

        if success == 0 {
            return Err("Failed to query token elevation information".to_string());
        }

        Ok(elevation.TokenIsElevated != 0)
    }
}

#[cfg(windows)]
pub use win::*;

#[cfg(not(windows))]
mod non_win {
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

#[cfg(not(windows))]
pub use non_win::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_battery_status_runs_without_panicking() {
        let result = query_battery_status();
        assert!(result.is_ok(), "query_battery_status returned an error: {:?}", result.err());
    }

    #[test]
    #[cfg(windows)]
    fn test_query_fast_startup_enabled_returns_boolean() {
        let result = query_fast_startup_enabled();
        assert!(result.is_ok(), "Failed to query fast startup status: {:?}", result.err());
    }

    #[test]
    #[cfg(windows)]
    fn test_query_is_admin_returns_boolean() {
        let result = query_is_admin();
        assert!(result.is_ok(), "Failed to query admin elevation status: {:?}", result.err());
    }

    #[test]
    #[cfg(windows)]
    fn test_query_domain_or_workgroup_returns_value() {
        let domain = query_domain_or_workgroup().expect("Failed to query domain or workgroup");
        assert!(!domain.is_empty(), "Domain or workgroup should not be empty");
    }

    #[test]
    #[cfg(windows)]
    fn test_query_build_number_returns_valid_build() {
        let build = query_build_number().expect("Failed to query build number");
        assert!(!build.is_empty(), "Build number should not be empty");
        let parsed: u32 = build.parse().expect("Build number should be numeric");
        assert!(parsed > 1000, "Build number should be > 1000");
    }

    #[test]
    #[cfg(windows)]
    fn test_query_motherboard_returns_info() {
        let board = query_motherboard().expect("Failed to query motherboard");
        assert!(board.is_some(), "Motherboard info should be detected on Windows");
        let b = board.unwrap();
        assert!(!b.manufacturer.is_empty(), "Manufacturer should not be empty");
    }

    #[test]
    #[cfg(windows)]
    fn test_query_gpus_detects_adapters() {
        let gpus = query_gpus().expect("Failed to query GPUs");
        assert!(!gpus.is_empty(), "At least one GPU adapter should be detected on Windows");
        for gpu in &gpus {
            assert!(!gpu.name.is_empty(), "GPU name should not be empty");
        }
    }
}

