pub mod system;
pub mod terminal;
pub mod types;
pub mod updates;

use types::{
    EndpointMachineInfo, OsSupportInfo, ShellDetectionResult, TerminalCommandResult,
    UpdateCheckResult, UpdateOperationResult,
};

#[tauri::command]
pub fn endpoint_check_os_support() -> OsSupportInfo {
    system::check_os_support()
}

#[tauri::command]
pub fn endpoint_get_machine_info() -> Result<EndpointMachineInfo, String> {
    system::gather_machine_info()
}

#[tauri::command]
pub fn endpoint_check_updates() -> Result<UpdateCheckResult, String> {
    updates::check_for_updates()
}

#[tauri::command]
pub fn endpoint_download_updates(
    target_id: Option<String>,
) -> Result<UpdateOperationResult, String> {
    updates::download_updates(target_id)
}

#[tauri::command]
pub fn endpoint_install_updates(
    target_id: Option<String>,
) -> Result<UpdateOperationResult, String> {
    updates::install_updates(target_id)
}

#[tauri::command]
pub fn endpoint_detect_shell() -> ShellDetectionResult {
    terminal::detect_shell()
}

#[tauri::command]
pub fn endpoint_execute_command(
    command: String,
    working_dir: Option<String>,
) -> Result<TerminalCommandResult, String> {
    terminal::execute_terminal_command(&command, working_dir)
}
