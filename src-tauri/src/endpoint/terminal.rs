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

pub fn start_terminal_session(
    app: AppHandle,
    cols: u16,
    rows: u16,
) -> Result<ShellDetectionResult, String> {
    if !cfg!(target_os = "windows") {
        return Err("Terminal is only supported on Windows.".to_string());
    }

    let shell = detect_shell();
    let exec = shell.executable_path.clone();

    // Kill any existing session before starting a fresh one
    close_terminal_session();

    let pty_system = native_pty_system();
    let size = PtySize {
        rows: rows.max(10),
        cols: cols.max(20),
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system
        .openpty(size)
        .map_err(|e| format!("Failed to open ConPTY pseudo-terminal: {e}"))?;

    let mut cmd = CommandBuilder::new(&exec);
    cmd.args(["-NoLogo"]);

    if let Ok(home) = std::env::var("USERPROFILE") {
        cmd.cwd(home);
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

    // Spawn reader thread that streams output in real-time to xterm frontend
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
        let _ = app.emit("terminal://exit", ());
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
    let mut active = ACTIVE_SESSION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *active = None;
}
