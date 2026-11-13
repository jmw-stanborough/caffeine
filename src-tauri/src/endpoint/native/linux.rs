use super::super::types::{BatteryInfo, GpuInfo, MotherboardInfo};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

pub fn query_is_admin() -> Result<bool, String> {
    super::query_unix_is_admin()
}

pub fn query_battery_status() -> Result<Option<BatteryInfo>, String> {
    query_battery_status_from_dir(Path::new("/sys/class/power_supply"))
}

pub fn query_battery_status_from_dir(dir: &Path) -> Result<Option<BatteryInfo>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Failed to read {}: {e}", dir.display())),
    };

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry in {}: {e}", dir.display()))?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();

    for name in names {
        let supply_dir = dir.join(&name);
        let is_bat_name = name.starts_with("BAT");
        let is_bat_type = read_optional_trimmed(&supply_dir.join("type"))?
            .is_some_and(|t| t.eq_ignore_ascii_case("Battery"));

        if !is_bat_name && !is_bat_type {
            continue;
        }

        // Skip peripheral device batteries (e.g. wireless mice/keyboards)
        if let Some(scope) = read_optional_trimmed(&supply_dir.join("scope"))?
            && scope.eq_ignore_ascii_case("Device")
        {
            continue;
        }

        if let Some(present) = read_optional_trimmed(&supply_dir.join("present"))?
            && present == "0"
        {
            continue;
        }

        let Some(capacity_raw) = read_optional_trimmed(&supply_dir.join("capacity"))? else {
            continue;
        };
        let Some(status_raw) = read_optional_trimmed(&supply_dir.join("status"))? else {
            continue;
        };

        return parse_battery_sysfs(&capacity_raw, &status_raw).map(Some);
    }

    Ok(None)
}

pub fn parse_battery_sysfs(capacity_raw: &str, status_raw: &str) -> Result<BatteryInfo, String> {
    let charge_percent = capacity_raw
        .trim()
        .parse::<u8>()
        .map_err(|e| format!("Invalid battery capacity '{}': {e}", capacity_raw.trim()))?
        .min(100);

    let status = status_raw.trim();
    let is_charging =
        status.eq_ignore_ascii_case("Charging") || status.eq_ignore_ascii_case("Full");

    Ok(BatteryInfo {
        has_battery: true,
        is_charging,
        charge_percent,
    })
}

pub fn query_domain_or_workgroup() -> Result<String, String> {
    if let Ok(domain) = crate::process::run_command_stdout("hostname", &["-d"])
        && !domain.is_empty()
        && domain != "(none)"
    {
        return Ok(domain);
    }

    match fs::read_to_string("/etc/resolv.conf") {
        Ok(contents) => {
            if let Some(domain) = parse_resolv_conf_domain(&contents) {
                return Ok(domain);
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(format!("Failed to read /etc/resolv.conf: {e}")),
    }

    Ok("WORKGROUP".to_string())
}

pub fn parse_resolv_conf_domain(contents: &str) -> Option<String> {
    let mut search_domain: Option<String> = None;

    for raw_line in contents.lines() {
        let line = raw_line
            .split(['#', ';'])
            .next()
            .unwrap_or("")
            .trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(directive) = parts.next() else {
            continue;
        };

        if directive == "domain" {
            if let Some(val) = parts.next()
                && !val.is_empty()
                && val != "."
            {
                return Some(val.to_string());
            }
        } else if directive == "search" && search_domain.is_none() {
            for val in parts {
                if !val.is_empty() && val != "." {
                    search_domain = Some(val.to_string());
                    break;
                }
            }
        }
    }

    search_domain
}

pub fn query_build_number() -> Result<String, String> {
    Ok(sysinfo::System::kernel_version().unwrap_or_default())
}

pub fn query_motherboard() -> Result<Option<MotherboardInfo>, String> {
    query_motherboard_from_dir(Path::new("/sys/devices/virtual/dmi/id"))
}

pub fn query_motherboard_from_dir(dir: &Path) -> Result<Option<MotherboardInfo>, String> {
    if !dir.exists() {
        return Ok(None);
    }

    let manufacturer = read_dmi_with_fallback(dir, "board_vendor", "sys_vendor")?;
    let product = read_dmi_with_fallback(dir, "board_name", "product_name")?;
    let bios_vendor = read_optional_trimmed(&dir.join("bios_vendor"))?.unwrap_or_default();
    let bios_version = read_optional_trimmed(&dir.join("bios_version"))?.unwrap_or_default();
    let bios_date = read_optional_trimmed(&dir.join("bios_date"))?.unwrap_or_default();

    if manufacturer.is_empty()
        && product.is_empty()
        && bios_vendor.is_empty()
        && bios_version.is_empty()
        && bios_date.is_empty()
    {
        return Ok(None);
    }

    Ok(Some(MotherboardInfo {
        manufacturer,
        product,
        bios_vendor,
        bios_version,
        bios_date,
    }))
}

fn read_dmi_with_fallback(dir: &Path, primary: &str, fallback: &str) -> Result<String, String> {
    if let Some(val) = read_optional_trimmed(&dir.join(primary))?
        && !val.is_empty()
    {
        return Ok(val);
    }
    Ok(read_optional_trimmed(&dir.join(fallback))?.unwrap_or_default())
}

pub fn query_gpus() -> Result<Vec<GpuInfo>, String> {
    let lspci_map = query_lspci_gpu_map();
    query_gpus_from_sysfs(Path::new("/sys/class/drm"), &lspci_map)
}

pub fn query_gpus_from_sysfs(
    drm_dir: &Path,
    lspci_map: &HashMap<String, String>,
) -> Result<Vec<GpuInfo>, String> {
    let entries = match fs::read_dir(drm_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("Failed to read {}: {e}", drm_dir.display())),
    };

    let mut card_names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry in {}: {e}", drm_dir.display()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_primary_drm_card(&name) {
            card_names.push(name);
        }
    }
    card_names.sort();

    let mut gpus = Vec::new();
    for card in card_names {
        let device_dir = drm_dir.join(&card).join("device");
        let Some(uevent) = read_optional_trimmed(&device_dir.join("uevent"))? else {
            continue;
        };

        let (driver, pci_slot) = parse_drm_uevent(&uevent);
        let vram_bytes = match read_optional_trimmed(&device_dir.join("mem_info_vram_total"))? {
            Some(raw) => raw
                .parse::<u64>()
                .map_err(|e| format!("Invalid VRAM size '{raw}' for {card}: {e}"))?,
            None => 0,
        };

        let driver_str = driver.unwrap_or_default();
        let adapter_name = pci_slot
            .as_deref()
            .and_then(|slot| lookup_pci_name(lspci_map, slot))
            .unwrap_or_else(|| {
                if driver_str.is_empty() {
                    format!("Display Adapter ({card})")
                } else {
                    format!("{driver_str} Display Adapter")
                }
            });

        gpus.push(GpuInfo {
            name: adapter_name,
            driver_version: driver_str,
            vram_bytes,
            temperature_celsius: None,
        });
    }

    Ok(gpus)
}

pub fn is_primary_drm_card(name: &str) -> bool {
    name.strip_prefix("card")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}

pub fn parse_drm_uevent(contents: &str) -> (Option<String>, Option<String>) {
    let mut driver = None;
    let mut pci_slot = None;

    for line in contents.lines() {
        if let Some(val) = line.strip_prefix("DRIVER=") {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                driver = Some(trimmed.to_string());
            }
        } else if let Some(val) = line.strip_prefix("PCI_SLOT_NAME=") {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                pci_slot = Some(trimmed.to_string());
            }
        }
    }

    (driver, pci_slot)
}

fn query_lspci_gpu_map() -> HashMap<String, String> {
    let Ok(output) = Command::new("lspci").arg("-D").output() else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }
    parse_lspci_gpu_map(&String::from_utf8_lossy(&output.stdout))
}

pub fn parse_lspci_gpu_map(stdout: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Some((slot, rest)) = trimmed.split_once(' ') else {
            continue;
        };
        let Some((class, model)) = rest.split_once(": ") else {
            continue;
        };

        let class_lower = class.to_lowercase();
        if class_lower.contains("vga")
            || class_lower.contains("3d")
            || class_lower.contains("display")
        {
            let clean_model = model.trim().to_string();
            if !clean_model.is_empty() {
                map.insert(slot.trim().to_string(), clean_model);
            }
        }
    }

    map
}

fn lookup_pci_name(lspci_map: &HashMap<String, String>, slot: &str) -> Option<String> {
    if let Some(name) = lspci_map.get(slot) {
        return Some(name.clone());
    }
    // Fall back to matching without the 4-digit PCI domain prefix (e.g. "0000:03:00.0" -> "03:00.0")
    if let Some((_, short_slot)) = slot.split_once(':')
        && let Some(name) = lspci_map.get(short_slot)
    {
        return Some(name.clone());
    }
    None
}

pub fn query_fast_startup_enabled() -> Result<bool, String> {
    Ok(false)
}

fn read_optional_trimmed(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(contents.trim().to_string())),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Failed to read {}: {e}", path.display())),
    }
}
