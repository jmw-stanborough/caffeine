export interface OsSupportInfo {
  is_windows: boolean;
  os_name: string;
  os_version: string;
  architecture: string;
  hostname: string;
  error_message?: string;
  is_admin: boolean;
}

export interface CpuInfo {
  model_name: string;
  vendor_id: string;
  physical_cores: number;
  logical_cores: number;
  frequency_mhz: number;
  global_usage_percent: number;
  core_usages: number[];
  temperature_celsius?: number;
}

export interface MemoryInfo {
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  available_bytes: number;
  usage_percent: number;
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  file_system: string;
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  usage_percent: number;
  is_removable: boolean;
  disk_type: string;
}

export interface GpuInfo {
  name: string;
  driver_version: string;
  vram_bytes: number;
  temperature_celsius?: number;
}

export interface TemperatureReading {
  label: string;
  celsius: number;
  max_celsius?: number;
  critical_celsius?: number;
}

export interface NetworkInterfaceInfo {
  name: string;
  mac_address: string;
  ip_v4: string[];
  ip_v6: string[];
  is_up: boolean;
  bytes_received: number;
  bytes_transmitted: number;
}

export interface MotherboardInfo {
  manufacturer: string;
  product: string;
  bios_vendor: string;
  bios_version: string;
  bios_date: string;
}

export interface SystemMetaInfo {
  computer_name: string;
  domain_or_workgroup: string;
  os_name: string;
  os_version: string;
  build_number: string;
  architecture: string;
  uptime_seconds: number;
  boot_timestamp: number;
  fast_startup_enabled: boolean;
}

export interface BatteryInfo {
  has_battery: boolean;
  is_charging: boolean;
  charge_percent: number;
}

export interface EndpointMachineInfo {
  system: SystemMetaInfo;
  cpu: CpuInfo;
  memory: MemoryInfo;
  disks: DiskInfo[];
  gpus: GpuInfo[];
  temperatures: TemperatureReading[];
  networks: NetworkInterfaceInfo[];
  motherboard?: MotherboardInfo;
  battery?: BatteryInfo;
}

export interface SystemUpdateItem {
  id: string;
  title: string;
  kb_article_id?: string;
  description?: string;
  size_bytes?: number;
  categories: string[];
  severity?: string;
  is_mandatory: boolean;
  is_downloaded: boolean;
  support_url?: string;
}

export interface UpdateCheckResult {
  updates: SystemUpdateItem[];
  reboot_required: boolean;
  last_checked: string;
}

export interface UpdateOperationResult {
  success: boolean;
  reboot_required: boolean;
  message: string;
  succeeded_count: number;
  failed_count: number;
}

export interface ShellDetectionResult {
  shell_name: string;
  executable_path: string;
  version: string;
  is_pwsh: boolean;
}

export interface TerminalCommandResult {
  command: string;
  exit_code: number;
  stdout: string;
  stderr: string;
  working_dir: string;
}

export interface TerminalEntry {
  id: string;
  command: string;
  working_dir: string;
  timestamp: string;
  result?: TerminalCommandResult;
  is_running?: boolean;
}
