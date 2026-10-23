use std::process::Command;
use std::sync::{LazyLock, Mutex};
use sysinfo::{Components, Disks, Networks, System};

use super::types::{
    BatteryInfo, CpuInfo, DiskInfo, EndpointMachineInfo, GpuInfo, MemoryInfo, MotherboardInfo,
    NetworkInterfaceInfo, OsSupportInfo, SystemMetaInfo, TemperatureReading,
};

static PERSISTENT_SYSTEM: LazyLock<Mutex<System>> =
    LazyLock::new(|| Mutex::new(System::new_all()));

static CACHED_STATIC_HARDWARE: LazyLock<Mutex<Option<StaticHardwareInfo>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Clone)]
struct StaticHardwareInfo {
    domain: String,
    motherboard: Option<MotherboardInfo>,
    gpus: Vec<GpuInfo>,
    build_number: String,
}

pub fn check_os_support() -> OsSupportInfo {
    let is_windows = cfg!(target_os = "windows");
    let os_name = System::name().unwrap_or_else(|| std::env::consts::OS.to_string());
    let os_version = System::os_version().unwrap_or_default();
    let architecture = System::cpu_arch();
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

    let error_message = if !is_windows {
        Some(format!(
            "Unsupported Operating System: {os_name} ({architecture}). Caffeine Endpoint is only supported on Microsoft Windows."
        ))
    } else {
        None
    };

    OsSupportInfo {
        is_windows,
        os_name,
        os_version,
        architecture,
        hostname,
        error_message,
    }
}

pub fn gather_machine_info() -> Result<EndpointMachineInfo, String> {
    if !cfg!(target_os = "windows") {
        return Err("The Endpoint module is currently only supported on Windows.".to_string());
    }

    let (
        model_name,
        vendor_id,
        frequency_mhz,
        physical_cores,
        logical_cores,
        global_usage_percent,
        core_usages,
        total_bytes,
        used_bytes,
        free_bytes,
        available_bytes,
        usage_percent,
    ) = {
        let mut sys = PERSISTENT_SYSTEM
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let cpus = sys.cpus();
        let model = cpus
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Unknown Processor".to_string());
        let vendor = cpus
            .first()
            .map(|c| c.vendor_id().trim().to_string())
            .unwrap_or_default();
        let freq = cpus.first().map(|c| c.frequency()).unwrap_or(0);
        let phys_cores = System::physical_core_count().unwrap_or(cpus.len());
        let log_cores = cpus.len();
        let global_usage = sys.global_cpu_usage();
        let core_u: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();

        let total = sys.total_memory();
        let used = sys.used_memory();
        let free = sys.free_memory();
        let available = sys.available_memory();
        let usage = if total > 0 {
            ((used as f64 / total as f64) * 100.0) as f32
        } else {
            0.0
        };

        (
            model,
            vendor,
            freq,
            phys_cores,
            log_cores,
            global_usage,
            core_u,
            total,
            used,
            free,
            available,
            usage,
        )
    };

    let memory = MemoryInfo {
        total_bytes,
        used_bytes,
        free_bytes,
        available_bytes,
        usage_percent,
    };

    let disks_list = Disks::new_with_refreshed_list();
    let mut disks = Vec::new();
    for d in &disks_list {
        let total = d.total_space();
        let available = d.available_space();
        let used = total.saturating_sub(available);
        let usage = if total > 0 {
            ((used as f64 / total as f64) * 100.0) as f32
        } else {
            0.0
        };

        let mount = d.mount_point().to_string_lossy().to_string();
        let name_str = d.name().to_string_lossy().to_string();
        let display_name = if name_str.is_empty() {
            mount.clone()
        } else {
            format!("{name_str} ({mount})")
        };

        disks.push(DiskInfo {
            name: display_name,
            mount_point: mount,
            file_system: d.file_system().to_string_lossy().to_string(),
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percent: usage,
            is_removable: d.is_removable(),
            disk_type: format!("{:?}", d.kind()),
        });
    }

    let mut temperatures = Vec::new();
    let components = Components::new_with_refreshed_list();
    for c in &components {
        if let Some(temp) = c.temperature() {
            temperatures.push(TemperatureReading {
                label: c.label().to_string(),
                celsius: temp,
                max_celsius: c.max(),
                critical_celsius: c.critical(),
            });
        }
    }

    let networks_list = Networks::new_with_refreshed_list();
    let mut networks = Vec::new();
    for (name, net) in &networks_list {
        let mac = net.mac_address().to_string();
        if mac == "00:00:00:00:00:00" && net.total_received() == 0 && net.total_transmitted() == 0 {
            continue;
        }

        let mut ip_v4 = Vec::new();
        let mut ip_v6 = Vec::new();
        for ip in net.ip_networks() {
            match ip.addr {
                std::net::IpAddr::V4(v4) => ip_v4.push(v4.to_string()),
                std::net::IpAddr::V6(v6) => ip_v6.push(v6.to_string()),
            }
        }

        networks.push(NetworkInterfaceInfo {
            name: name.clone(),
            mac_address: mac,
            ip_v4,
            ip_v6,
            is_up: net.total_received() > 0 || net.total_transmitted() > 0,
            bytes_received: net.total_received(),
            bytes_transmitted: net.total_transmitted(),
        });
    }

    let static_info = get_or_load_static_hardware();

    let system_meta = SystemMetaInfo {
        computer_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        domain_or_workgroup: static_info.domain,
        os_name: System::name().unwrap_or_else(|| "Windows".to_string()),
        os_version: System::os_version().unwrap_or_default(),
        build_number: static_info.build_number,
        architecture: System::cpu_arch(),
        uptime_seconds: System::uptime(),
        boot_timestamp: System::boot_time(),
    };

    let cpu = CpuInfo {
        model_name,
        vendor_id,
        physical_cores,
        logical_cores,
        frequency_mhz,
        global_usage_percent,
        core_usages,
        temperature_celsius: temperatures.first().map(|t| t.celsius),
    };

    let battery = query_battery_status();

    Ok(EndpointMachineInfo {
        system: system_meta,
        cpu,
        memory,
        disks,
        gpus: static_info.gpus,
        temperatures,
        networks,
        motherboard: static_info.motherboard,
        battery,
    })
}

fn get_or_load_static_hardware() -> StaticHardwareInfo {
    let mut cache = CACHED_STATIC_HARDWARE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(info) = &*cache {
        return info.clone();
    }

    let loaded = query_static_windows_hardware();
    *cache = Some(loaded.clone());
    loaded
}

fn query_static_windows_hardware() -> StaticHardwareInfo {
    let domain = query_powershell_scalar("(Get-CimInstance Win32_ComputerSystem).Domain")
        .unwrap_or_else(|_| "WORKGROUP".to_string());

    let gpus = query_gpus();
    let motherboard = query_motherboard();
    let build_number = query_powershell_scalar(
        "(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion').CurrentBuildNumber",
    )
    .unwrap_or_default();

    StaticHardwareInfo {
        domain,
        motherboard,
        gpus,
        build_number,
    }
}

fn query_gpus() -> Vec<GpuInfo> {
    let script = "@(Get-CimInstance Win32_VideoController | Select-Object Name, DriverVersion, AdapterRAM) | ConvertTo-Json -Compress";
    let output = match Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return Vec::new(),
    };

    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    #[derive(serde::Deserialize)]
    struct RawGpu {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "DriverVersion")]
        driver_version: Option<String>,
        #[serde(rename = "AdapterRAM")]
        adapter_ram: Option<u64>,
    }

    if let Ok(list) = serde_json::from_str::<Vec<RawGpu>>(trimmed) {
        list.into_iter()
            .map(|g| GpuInfo {
                name: g.name.unwrap_or_else(|| "Display Adapter".to_string()),
                driver_version: g.driver_version.unwrap_or_default(),
                vram_bytes: g.adapter_ram.unwrap_or(0),
                temperature_celsius: None,
            })
            .collect()
    } else if let Ok(single) = serde_json::from_str::<RawGpu>(trimmed) {
        vec![GpuInfo {
            name: single.name.unwrap_or_else(|| "Display Adapter".to_string()),
            driver_version: single.driver_version.unwrap_or_default(),
            vram_bytes: single.adapter_ram.unwrap_or(0),
            temperature_celsius: None,
        }]
    } else {
        Vec::new()
    }
}

fn query_motherboard() -> Option<MotherboardInfo> {
    let script = "$b = Get-CimInstance Win32_BaseBoard; $bios = Get-CimInstance Win32_BIOS; [PSCustomObject]@{ Manufacturer = $b.Manufacturer; Product = $b.Product; BiosVendor = $bios.Manufacturer; BiosVersion = $bios.SMBIOSBIOSVersion; BiosDate = [string]$bios.ReleaseDate } | ConvertTo-Json -Compress";
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;

    let trimmed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if trimmed.is_empty() {
        return None;
    }

    #[derive(serde::Deserialize)]
    struct RawBoard {
        #[serde(rename = "Manufacturer")]
        manufacturer: Option<String>,
        #[serde(rename = "Product")]
        product: Option<String>,
        #[serde(rename = "BiosVendor")]
        bios_vendor: Option<String>,
        #[serde(rename = "BiosVersion")]
        bios_version: Option<String>,
        #[serde(rename = "BiosDate")]
        bios_date: Option<String>,
    }

    let raw: RawBoard = serde_json::from_str(&trimmed).ok()?;
    Some(MotherboardInfo {
        manufacturer: raw.manufacturer.unwrap_or_default(),
        product: raw.product.unwrap_or_default(),
        bios_vendor: raw.bios_vendor.unwrap_or_default(),
        bios_version: raw.bios_version.unwrap_or_default(),
        bios_date: raw.bios_date.unwrap_or_default(),
    })
}

fn query_battery_status() -> Option<BatteryInfo> {
    let script = "$bat = Get-CimInstance Win32_Battery -ErrorAction SilentlyContinue; if ($bat) { [PSCustomObject]@{ HasBattery = $true; IsCharging = ($bat.BatteryStatus -eq 2); ChargePercent = [int]$bat.EstimatedChargeRemaining } | ConvertTo-Json -Compress }";
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;

    let trimmed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if trimmed.is_empty() {
        return None;
    }

    #[derive(serde::Deserialize)]
    struct RawBattery {
        #[serde(rename = "HasBattery")]
        has_battery: Option<bool>,
        #[serde(rename = "IsCharging")]
        is_charging: Option<bool>,
        #[serde(rename = "ChargePercent")]
        charge_percent: Option<u8>,
    }

    let raw: RawBattery = serde_json::from_str(&trimmed).ok()?;
    Some(BatteryInfo {
        has_battery: raw.has_battery.unwrap_or(false),
        is_charging: raw.is_charging.unwrap_or(false),
        charge_percent: raw.charge_percent.unwrap_or(0),
    })
}

fn query_powershell_scalar(script: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| format!("PowerShell execution failed: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
