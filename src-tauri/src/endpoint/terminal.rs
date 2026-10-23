use std::process::Command;
use std::sync::{LazyLock, Mutex};
use super::types::{ShellDetectionResult, TerminalCommandResult};

static DETECTED_SHELL: LazyLock<Mutex<Option<ShellDetectionResult>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn detect_shell() -> ShellDetectionResult {
    let mut cache = DETECTED_SHELL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(cached) = &*cache {
        return cached.clone();
    }

    let detected = detect_shell_internal();
    *cache = Some(detected.clone());
    detected
}

fn detect_shell_internal() -> ShellDetectionResult {
    if !cfg!(target_os = "windows") {
        return ShellDetectionResult {
            shell_name: "unsupported".to_string(),
            executable_path: String::new(),
            version: "N/A".to_string(),
            is_pwsh: false,
        };
    }

    if let Ok(output) = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$PSVersionTable.PSVersion.ToString()",
        ])
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return ShellDetectionResult {
                shell_name: "pwsh".to_string(),
                executable_path: "pwsh".to_string(),
                version,
                is_pwsh: true,
            };
        }
    }

    let ps_output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$PSVersionTable.PSVersion.ToString()",
        ])
        .output();

    let version = ps_output
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "5.1".to_string());

    ShellDetectionResult {
        shell_name: "powershell".to_string(),
        executable_path: "powershell".to_string(),
        version,
        is_pwsh: false,
    }
}

pub fn execute_terminal_command(
    command_str: &str,
    working_dir: Option<String>,
) -> Result<TerminalCommandResult, String> {
    if !cfg!(target_os = "windows") {
        return Err("Terminal is only supported on Windows.".to_string());
    }

    let shell = detect_shell();
    let exec = shell.executable_path;

    let initial_cwd = working_dir
        .filter(|d| !d.trim().is_empty())
        .unwrap_or_else(|| {
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string())
        });

    let wrapped_script = format!(
        r#"
$ProgressPreference = 'SilentlyContinue'
Set-Location -LiteralPath '{initial_cwd}'
try {{
    {command_str}
}} finally {{
    [Console]::Out.Flush()
    Write-Host ('__CAFFEINE_PWD__:' + (Get-Location).Path)
}}
"#
    );

    let output = Command::new(&exec)
        .args(["-NoProfile", "-NonInteractive", "-Command", &wrapped_script])
        .output()
        .map_err(|e| format!("Failed to execute terminal process {exec}: {e}"))?;

    let raw_stdout = String::from_utf8_lossy(&output.stdout);
    let raw_stderr = String::from_utf8_lossy(&output.stderr);

    let mut clean_stdout_lines = Vec::new();
    let mut updated_dir = initial_cwd;

    for line in raw_stdout.lines() {
        if let Some(pwd_suffix) = line.strip_prefix("__CAFFEINE_PWD__:") {
            let candidate = pwd_suffix.trim();
            if !candidate.is_empty() {
                updated_dir = candidate.to_string();
            }
        } else {
            clean_stdout_lines.push(line);
        }
    }

    let exit_code = output.status.code().unwrap_or(if output.status.success() { 0 } else { 1 });

    Ok(TerminalCommandResult {
        command: command_str.to_string(),
        exit_code,
        stdout: clean_stdout_lines.join("\n"),
        stderr: raw_stderr.trim().to_string(),
        working_dir: updated_dir,
    })
}
