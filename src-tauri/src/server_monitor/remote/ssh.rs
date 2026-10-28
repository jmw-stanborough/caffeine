use super::validation::{resolve_port, sanitize_host, sanitize_username};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Emitter};

const DEFAULT_SSH_PORT: u16 = 22;

struct ActiveSshSession {
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
}

static ACTIVE_SSH_SESSIONS: LazyLock<Mutex<HashMap<String, ActiveSshSession>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn start_ssh_session(
    app: AppHandle,
    server_id: &str,
    host: &str,
    port: Option<u16>,
    username: Option<&str>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let sanitized_host = sanitize_host(host)?;
    let sanitized_user = sanitize_username(username)?;
    let effective_port = resolve_port(port, DEFAULT_SSH_PORT)?;

    close_ssh_session(server_id);

    let pty_system = native_pty_system();
    let size = PtySize {
        rows: rows.max(5),
        cols: cols.max(10),
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system
        .openpty(size)
        .map_err(|e| format!("Failed to open PTY system: {e}"))?;

    let mut cmd = CommandBuilder::new("ssh");
    cmd.args(["-p", &effective_port.to_string()]);

    let destination = match sanitized_user {
        Some(user) => format!("{user}@{sanitized_host}"),
        None => sanitized_host,
    };
    cmd.arg(&destination);

    pair.slave
        .spawn_command(cmd)
        .map_err(|e| format!("Failed to spawn SSH process: {e}"))?;

    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("Failed to clone PTY reader: {e}"))?;

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("Failed to obtain PTY writer: {e}"))?;

    let session = ActiveSshSession {
        writer,
        master: pair.master,
    };

    {
        let mut sessions = ACTIVE_SSH_SESSIONS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        sessions.insert(server_id.to_string(), session);
    }

    let sid = server_id.to_string();
    std::thread::spawn(move || {
        let data_event = format!("server-terminal://{sid}/data");
        let exit_event = format!("server-terminal://{sid}/exit");
        let mut buf = [0u8; 2048];

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    if app.emit(&data_event, chunk).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        let _ = app.emit(&exit_event, ());
        close_ssh_session(&sid);
    });

    Ok(())
}

pub fn write_to_ssh_session(server_id: &str, data: &str) -> Result<(), String> {
    let mut sessions = ACTIVE_SSH_SESSIONS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(session) = sessions.get_mut(server_id) {
        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("Failed to write to SSH session: {e}"))?;
        session
            .writer
            .flush()
            .map_err(|e| format!("Failed to flush SSH input: {e}"))?;
        Ok(())
    } else {
        Err(format!("No active SSH session for server {server_id}"))
    }
}

pub fn resize_ssh_session(server_id: &str, cols: u16, rows: u16) -> Result<(), String> {
    let mut sessions = ACTIVE_SSH_SESSIONS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(session) = sessions.get_mut(server_id) {
        let size = PtySize {
            rows: rows.max(5),
            cols: cols.max(10),
            pixel_width: 0,
            pixel_height: 0,
        };
        session
            .master
            .resize(size)
            .map_err(|e| format!("Failed to resize SSH terminal: {e}"))?;
        Ok(())
    } else {
        Ok(())
    }
}

pub fn close_ssh_session(server_id: &str) {
    let mut sessions = ACTIVE_SSH_SESSIONS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    sessions.remove(server_id);
}
