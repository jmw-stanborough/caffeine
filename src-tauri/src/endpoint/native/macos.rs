use super::super::types::{BatteryInfo, GpuInfo, MotherboardInfo};
use serde_json::Value;

pub fn query_is_admin() -> Result<bool, String> {
    super::query_unix_is_admin()
}

pub fn query_battery_status() -> Result<Option<BatteryInfo>, String> {
    let stdout = crate::process::run_command_stdout("pmset", &["-g", "batt"])?;
    parse_pmset_batt(&stdout)
}

pub fn parse_pmset_batt(stdout: &str) -> Result<Option<BatteryInfo>, String> {
    let Some(batt_line) = stdout.lines().find(|line| line.contains("InternalBattery")) else {
        return Ok(None);
    };

    if batt_line.contains("present: false") {
        return Ok(None);
    }

    let pct_end = batt_line
        .find('%')
        .ok_or_else(|| format!("Missing battery percentage in pmset output: {batt_line}"))?;
    let prefix = &batt_line[..pct_end];
    let digits_start = prefix
        .rfind(|c: char| !c.is_ascii_digit())
        .map_or(0, |idx| idx + 1);
    let charge_percent = prefix[digits_start..]
        .parse::<u8>()
        .map_err(|e| format!("Invalid battery percentage in pmset output '{batt_line}': {e}"))?
        .min(100);

    let has_ac_power = stdout.contains("AC Power");
    let states: Vec<&str> = batt_line
        .split(';')
        .skip(1)
        .map(str::trim)
        .collect();
    let is_discharging = states
        .iter()
        .any(|s| s.eq_ignore_ascii_case("discharging"));
    let is_charging_state = states.iter().any(|s| {
        s.eq_ignore_ascii_case("charging")
            || s.eq_ignore_ascii_case("charged")
            || s.eq_ignore_ascii_case("finishing charge")
    });

    let is_charging = !is_discharging && (is_charging_state || has_ac_power);

    Ok(Some(BatteryInfo {
        has_battery: true,
        is_charging,
        charge_percent,
    }))
}

pub fn query_domain_or_workgroup() -> Result<String, String> {
    if let Ok(dns_out) = crate::process::run_command_stdout("scutil", &["--dns"])
        && let Some(domain) = parse_scutil_dns_domain(&dns_out)
    {
        return Ok(domain);
    }

    if let Ok(host_out) = crate::process::run_command_stdout("hostname", &[])
        && let Some(domain) = parse_hostname_domain(&host_out)
    {
        return Ok(domain);
    }

    Ok("LOCAL".to_string())
}

pub fn parse_scutil_dns_domain(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("search domain")
            && let Some((_, val)) = trimmed.split_once(':')
        {
            let domain = val.trim();
            if !domain.is_empty() && domain != "." {
                return Some(domain.to_string());
            }
        }
    }
    None
}

pub fn parse_hostname_domain(hostname: &str) -> Option<String> {
    hostname
        .trim()
        .split_once('.')
        .map(|(_, domain)| domain.trim())
        .filter(|d| !d.is_empty())
        .map(str::to_string)
}

pub fn query_build_number() -> Result<String, String> {
    crate::process::run_command_stdout("sw_vers", &["-buildVersion"])
}

pub fn query_motherboard() -> Result<Option<MotherboardInfo>, String> {
    let stdout = crate::process::run_command_stdout("system_profiler", &["SPHardwareDataType", "-json"])?;
    parse_macos_hardware_json(&stdout)
}

pub fn parse_macos_hardware_json(json_str: &str) -> Result<Option<MotherboardInfo>, String> {
    let root: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse SPHardwareDataType JSON: {e}"))?;

    let Some(entry) = root
        .get("SPHardwareDataType")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
    else {
        return Ok(None);
    };

    let product = first_non_empty_str(entry, &["model_identifier", "machine_model", "machine_name"])
        .unwrap_or_default();
    let bios_version =
        first_non_empty_str(entry, &["boot_rom_version", "system_firmware_version"])
            .unwrap_or_default();

    Ok(Some(MotherboardInfo {
        manufacturer: "Apple Inc.".to_string(),
        product,
        bios_vendor: "Apple Inc.".to_string(),
        bios_version,
        bios_date: String::new(),
    }))
}

pub fn query_gpus() -> Result<Vec<GpuInfo>, String> {
    let stdout = crate::process::run_command_stdout("system_profiler", &["SPDisplaysDataType", "-json"])?;
    parse_macos_displays_json(&stdout)
}

pub fn parse_macos_displays_json(json_str: &str) -> Result<Vec<GpuInfo>, String> {
    let root: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse SPDisplaysDataType JSON: {e}"))?;

    let Some(items) = root.get("SPDisplaysDataType").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    let mut gpus = Vec::new();
    for item in items {
        let name = first_non_empty_str(item, &["sppci_model", "_name"])
            .unwrap_or_else(|| "Apple GPU".to_string());

        let raw_driver = first_non_empty_str(
            item,
            &[
                "spdisplays_mtlgpufamilysupport",
                "spdisplays_metal",
                "spdisplays_rom-revision",
                "spdisplays_efi-version",
            ],
        )
        .unwrap_or_default();
        let driver_version = raw_driver
            .strip_prefix("spdisplays_")
            .unwrap_or(&raw_driver)
            .to_string();

        let vram_bytes = first_non_empty_str(item, &["spdisplays_vram", "spdisplays_vram_shared"])
            .and_then(|s| parse_vram_bytes(&s))
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

pub fn parse_vram_bytes(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    let mut parts = trimmed.split_whitespace();
    let amount = parts.next()?.parse::<u64>().ok()?;
    let unit = parts.next().unwrap_or("B").to_ascii_uppercase();

    let multiplier = match unit.as_str() {
        "GB" | "GIB" => 1024 * 1024 * 1024,
        "MB" | "MIB" => 1024 * 1024,
        "KB" | "KIB" => 1024,
        "B" => 1,
        _ => return None,
    };

    Some(amount.saturating_mul(multiplier))
}

pub fn query_fast_startup_enabled() -> Result<bool, String> {
    Ok(false)
}

fn first_non_empty_str(obj: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = obj.get(*key).and_then(Value::as_str) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}
