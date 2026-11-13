use super::validation::{resolve_port, sanitize_host, sanitize_username};
use std::process::Command;

const DEFAULT_RDP_PORT: u16 = 3389;

pub fn launch_rdp_client(
    host: &str,
    port: Option<u16>,
    username: Option<&str>,
) -> Result<(), String> {
    let sanitized_host = sanitize_host(host)?;
    let sanitized_user = sanitize_username(username)?;
    let effective_port = resolve_port(port, DEFAULT_RDP_PORT)?;

    #[cfg(target_os = "windows")]
    {
        launch_windows_rdp(&sanitized_host, effective_port, sanitized_user.as_deref())
    }

    #[cfg(target_os = "macos")]
    {
        launch_macos_rdp(&sanitized_host, effective_port, sanitized_user.as_deref())
    }

    #[cfg(target_os = "linux")]
    {
        launch_linux_rdp(&sanitized_host, effective_port, sanitized_user.as_deref())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = (sanitized_host, effective_port, sanitized_user);
        Err("Remote Desktop is not supported on this operating system".to_string())
    }
}

#[cfg(target_os = "windows")]
fn launch_windows_rdp(host: &str, port: u16, _username: Option<&str>) -> Result<(), String> {
    let target = format!("{host}:{port}");
    Command::new("mstsc")
        .arg(format!("/v:{target}"))
        .spawn()
        .map_err(|e| format!("Failed to spawn mstsc.exe: {e}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_macos_rdp(host: &str, port: u16, username: Option<&str>) -> Result<(), String> {
    let url = match username {
        Some(user) => format!("rdp://{user}@{host}:{port}"),
        None => format!("rdp://{host}:{port}"),
    };
    Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to launch macOS Remote Desktop handler: {e}"))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_linux_rdp(host: &str, port: u16, username: Option<&str>) -> Result<(), String> {
    let target = format!("{host}:{port}");

    if command_exists("remmina") {
        let uri = match username {
            Some(user) => format!("rdp://{user}@{target}"),
            None => format!("rdp://{target}"),
        };
        return Command::new("remmina")
            .args(["-c", &uri])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Failed to spawn remmina: {e}"));
    }

    for client in ["xfreerdp3", "sdl-freerdp", "xfreerdp"] {
        if command_exists(client) {
            let mut cmd = Command::new(client);
            cmd.arg(format!("/v:{target}"));
            if let Some(user) = username {
                cmd.arg(format!("/u:{user}"));
            }
            return cmd
                .spawn()
                .map(|_| ())
                .map_err(|e| format!("Failed to spawn {client}: {e}"));
        }
    }

    let uri = format!("rdp://{target}");
    Command::new("xdg-open")
        .arg(&uri)
        .spawn()
        .map(|_| ())
        .map_err(|e| {
            format!(
                "No native RDP client (remmina/xfreerdp3/sdl-freerdp/xfreerdp) found, and xdg-open failed: {e}"
            )
        })
}

#[cfg(target_os = "linux")]
fn command_exists(name: &str) -> bool {
    crate::process::find_executable_in_path(name).is_some()
}
