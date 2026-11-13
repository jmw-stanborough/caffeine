use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsSupportInfo {
    pub is_windows: bool,
    pub is_supported: bool,
    pub os_family: String,
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub hostname: String,
    pub error_message: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model_name: String,
    pub vendor_id: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub frequency_mhz: u64,
    pub global_usage_percent: f32,
    pub core_usages: Vec<f32>,
    pub temperature_celsius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
    pub is_removable: bool,
    pub disk_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub driver_version: String,
    pub vram_bytes: u64,
    pub temperature_celsius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub label: String,
    pub celsius: f32,
    pub max_celsius: Option<f32>,
    pub critical_celsius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub mac_address: String,
    pub ip_v4: Vec<String>,
    pub ip_v6: Vec<String>,
    pub is_up: bool,
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub bios_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetaInfo {
    pub computer_name: String,
    pub domain_or_workgroup: String,
    pub os_name: String,
    pub os_version: String,
    pub build_number: String,
    pub architecture: String,
    pub uptime_seconds: u64,
    pub boot_timestamp: u64,
    pub fast_startup_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub has_battery: bool,
    pub is_charging: bool,
    pub charge_percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMachineInfo {
    pub system: SystemMetaInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub gpus: Vec<GpuInfo>,
    pub temperatures: Vec<TemperatureReading>,
    pub networks: Vec<NetworkInterfaceInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub battery: Option<BatteryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUpdateItem {
    pub id: String,
    pub title: String,
    pub kb_article_id: Option<String>,
    pub description: Option<String>,
    pub size_bytes: Option<u64>,
    pub categories: Vec<String>,
    pub severity: Option<String>,
    pub is_mandatory: bool,
    pub is_downloaded: bool,
    pub support_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub updates: Vec<SystemUpdateItem>,
    pub reboot_required: bool,
    pub last_checked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOperationResult {
    pub success: bool,
    pub reboot_required: bool,
    pub message: String,
    pub succeeded_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressEvent {
    pub phase: String,
    pub percent_complete: u32,
    pub current_index: usize,
    pub total_count: usize,
    pub bytes_downloaded: Option<u64>,
    pub total_bytes: Option<u64>,
    pub current_update_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellDetectionResult {
    pub shell_name: String,
    pub executable_path: String,
    pub version: String,
    pub is_pwsh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub working_dir: String,
}
