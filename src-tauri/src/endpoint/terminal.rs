use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Emitter};

use super::types::ShellDetectionResult;

struct ActiveTerminalSession {
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
}

static ACTIVE_SESSION: LazyLock<Mutex<Option<ActiveTerminalSession>>> =
    LazyLock::new(|| Mutex::new(None));

static SESSION_ID: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

static DETECTED_SHELL: LazyLock<Mutex<Option<ShellDetectionResult>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn detect_shell() -> Result<ShellDetectionResult, String> {
    let mut cache = DETECTED_SHELL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(cached) = &*cache {
        return Ok(cached.clone());
    }

    let detected = detect_shell_internal()?;
    *cache = Some(detected.clone());
    Ok(detected)
}

fn detect_shell_internal() -> Result<ShellDetectionResult, String> {
    if cfg!(target_os = "windows") {
        detect_windows_shell()
    } else {
        detect_unix_shell()
    }
}

fn detect_windows_shell() -> Result<ShellDetectionResult, String> {
    let mut pwsh_cmd = Command::new("pwsh");
    pwsh_cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$PSVersionTable.PSVersion.ToString()",
    ]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        pwsh_cmd.creation_flags(CREATE_NO_WINDOW);
    }

    if let Ok(output) = pwsh_cmd.output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(ShellDetectionResult {
                shell_name: "pwsh".to_string(),
                executable_path: "pwsh".to_string(),
                version,
                is_pwsh: true,
            });
        }
    }

    let mut ps_cmd = Command::new("powershell");
    ps_cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$PSVersionTable.PSVersion.ToString()",
    ]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        ps_cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let ps_output = ps_cmd
        .output()
        .map_err(|e| format!("Failed to spawn PowerShell: {e}"))?;

    if !ps_output.status.success() {
        let stderr = String::from_utf8_lossy(&ps_output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!(
                "PowerShell detection exited with status {:?}",
                ps_output.status.code()
            )
        } else {
            format!("PowerShell detection failed: {stderr}")
        });
    }

    let version = String::from_utf8_lossy(&ps_output.stdout).trim().to_string();
    if version.is_empty() {
        return Err("PowerShell returned empty version string during detection".to_string());
    }

    Ok(ShellDetectionResult {
        shell_name: "powershell".to_string(),
        executable_path: "powershell".to_string(),
        version,
        is_pwsh: false,
    })
}

fn detect_unix_shell() -> Result<ShellDetectionResult, String> {
    let mut candidates = Vec::new();

    if let Ok(env_shell) = std::env::var("SHELL") {
        let trimmed = env_shell.trim();
        if !trimmed.is_empty() {
            candidates.push(trimmed.to_string());
        }
    }

    if cfg!(target_os = "macos") {
        candidates.push("/bin/zsh".to_string());
        candidates.push("/bin/bash".to_string());
    } else {
        candidates.push("/bin/bash".to_string());
        candidates.push("/usr/bin/bash".to_string());
        candidates.push("/bin/zsh".to_string());
    }
    candidates.push("/bin/sh".to_string());

    for path in candidates {
        let std_path = std::path::Path::new(&path);
        if !std_path.is_file() {
            continue;
        }

        let shell_name = std_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sh")
            .to_string();

        let version = Command::new(&path)
            .arg("--version")
            .output()
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    let first_line = String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if !first_line.is_empty() {
                        return Some(first_line);
                    }
                }
                None
            })
            .unwrap_or_else(|| "unknown".to_string());

        let is_pwsh = shell_name.eq_ignore_ascii_case("pwsh");

        return Ok(ShellDetectionResult {
            shell_name,
            executable_path: path,
            version,
            is_pwsh,
        });
    }

    Err("No supported shell found in environment or standard paths".to_string())
}

pub fn start_terminal_session(
    app: AppHandle,
    cols: u16,
    rows: u16,
) -> Result<ShellDetectionResult, String> {
    let shell = detect_shell()?;

    {
        let active = ACTIVE_SESSION
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if active.is_some() {
            drop(active);
            let _ = resize_terminal(cols, rows);
            return Ok(shell);
        }
    }

    let exec = shell.executable_path.clone();

    let session_id = {
        let mut id = SESSION_ID
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *id += 1;
        *id
    };

    let pty_system = native_pty_system();
    let size = PtySize {
        rows: rows.max(10),
        cols: cols.max(20),
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system
        .openpty(size)
        .map_err(|e| format!("Failed to open pseudo-terminal: {e}"))?;

    let mut cmd = CommandBuilder::new(&exec);

    if cfg!(target_os = "windows") {
        if shell.is_pwsh || shell.shell_name.eq_ignore_ascii_case("powershell") {
            cmd.args(["-NoLogo"]);
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            cmd.cwd(home);
        }
    } else {
        cmd.args(["-l"]);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        if let Ok(home) = std::env::var("HOME") {
            cmd.cwd(home);
        }
    }

    pair.slave
        .spawn_command(cmd)
        .map_err(|e| format!("Failed to spawn {exec} in PTY: {e}"))?;

    // Drop slave in parent process so EOF propagates when child terminates
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("Failed to clone PTY reader: {e}"))?;

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("Failed to obtain PTY writer: {e}"))?;

    let session = ActiveTerminalSession {
        writer,
        master: pair.master,
    };

    {
        let mut active = ACTIVE_SESSION
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *active = Some(session);
    }

    std::thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    if app.emit("terminal://data", chunk).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let is_current = {
            let mut id = SESSION_ID
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if *id == session_id {
                *id += 1;
                true
            } else {
                false
            }
        };
        if is_current {
            close_terminal_session();
            let _ = app.emit("terminal://exit", ());
        }
    });

    Ok(shell)
}

pub fn write_to_terminal(data: &str) -> Result<(), String> {
    let mut active = ACTIVE_SESSION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(session) = active.as_mut() {
        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("Failed to write to terminal: {e}"))?;
        session
            .writer
            .flush()
            .map_err(|e| format!("Failed to flush terminal input: {e}"))?;
        Ok(())
    } else {
        Err("No active terminal session".to_string())
    }
}

pub fn resize_terminal(cols: u16, rows: u16) -> Result<(), String> {
    let mut active = ACTIVE_SESSION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(session) = active.as_mut() {
        let size = PtySize {
            rows: rows.max(5),
            cols: cols.max(10),
            pixel_width: 0,
            pixel_height: 0,
        };
        session
            .master
            .resize(size)
            .map_err(|e| format!("Failed to resize terminal: {e}"))?;
        Ok(())
    } else {
        Ok(())
    }
}

pub fn close_terminal_session() {
    {
        let mut id = SESSION_ID
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *id += 1;
    }
    let mut active = ACTIVE_SESSION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *active = None;
}
