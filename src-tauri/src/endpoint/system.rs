use std::sync::{LazyLock, Mutex};
use sysinfo::{Components, Disks, Networks, System};

use super::native;
use super::types::{
    CpuInfo, DiskInfo, EndpointMachineInfo, GpuInfo, MemoryInfo, MotherboardInfo,
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
    fast_startup_enabled: bool,
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

    let static_info = get_or_load_static_hardware()?;

    let system_meta = SystemMetaInfo {
        computer_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        domain_or_workgroup: static_info.domain,
        os_name: System::name().unwrap_or_else(|| "Windows".to_string()),
        os_version: System::os_version().unwrap_or_default(),
        build_number: static_info.build_number,
        architecture: System::cpu_arch(),
        uptime_seconds: System::uptime(),
        boot_timestamp: System::boot_time(),
        fast_startup_enabled: static_info.fast_startup_enabled,
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

    let battery = native::query_battery_status()?;

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

fn get_or_load_static_hardware() -> Result<StaticHardwareInfo, String> {
    let mut cache = CACHED_STATIC_HARDWARE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(info) = &*cache {
        return Ok(info.clone());
    }

    let loaded = query_static_windows_hardware()?;
    *cache = Some(loaded.clone());
    Ok(loaded)
}

fn query_static_windows_hardware() -> Result<StaticHardwareInfo, String> {
    let domain = native::query_domain_or_workgroup()?;
    let gpus = native::query_gpus()?;
    let motherboard = native::query_motherboard()?;
    let build_number = native::query_build_number()?;
    let fast_startup_enabled = native::query_fast_startup_enabled()?;

    Ok(StaticHardwareInfo {
        domain,
        motherboard,
        gpus,
        build_number,
        fast_startup_enabled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gather_machine_info() {
        if !cfg!(target_os = "windows") {
            return;
        }

        let info = gather_machine_info().expect("gather_machine_info should succeed without error");
        assert!(!info.system.computer_name.is_empty());
        assert!(!info.cpu.model_name.is_empty());
        assert!(info.memory.total_bytes > 0);
    }
}
