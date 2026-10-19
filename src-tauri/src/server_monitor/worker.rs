use super::config::load_servers;
use super::types::{Agent, HealthStatus, Server, ServerStatusEvent};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::net::TcpStream;
use tokio::time::timeout;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(1000);
const PROBE_PORTS: &[u16] = &[445, 135, 80, 443, 53, 22, 3389];

#[cfg(target_os = "windows")]
async fn unprivileged_icmp_ping(host: &str) -> Option<u64> {
    let host = host.to_string();
    tokio::task::spawn_blocking(move || {
        use std::net::ToSocketAddrs;
        use winping::{Buffer, Pinger};

        let ip = if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            ip
        } else {
            format!("{host}:0").to_socket_addrs().ok()?.next()?.ip()
        };

        let pinger = Pinger::new().ok()?;
        let mut buffer = Buffer::new();

        match pinger.send(ip, &mut buffer) {
            Ok(rtt) => Some(rtt as u64),
            Err(_) => None,
        }
    })
    .await
    .ok()?
}

#[cfg(not(target_os = "windows"))]
async fn unprivileged_icmp_ping(_host: &str) -> Option<u64> {
    None
}

async fn probe_tcp(host: &str, custom_port: Option<u16>) -> Option<u64> {
    let mut ports = Vec::new();
    if let Some(port) = custom_port {
        ports.push(port);
    }
    ports.extend_from_slice(PROBE_PORTS);

    for port in ports {
        let start = Instant::now();
        let target = format!("{host}:{port}");

        match timeout(CONNECT_TIMEOUT, TcpStream::connect(&target)).await {
            Ok(Ok(_)) => return Some(start.elapsed().as_millis() as u64),
            Ok(Err(err)) => {
                if err.kind() == std::io::ErrorKind::ConnectionRefused {
                    return Some(start.elapsed().as_millis() as u64);
                }
            }
            Err(_) => continue,
        }
    }

    None
}

async fn check_server(server: &Server) -> ServerStatusEvent {
    let agent_port = server.agent.as_ref().map(|Agent { port, .. }| *port);

    if let Some(latency) = unprivileged_icmp_ping(&server.host).await {
        let status = if latency > 300 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Online
        };
        return ServerStatusEvent {
            host: server.host.clone(),
            status,
            latency_ms: Some(latency),
        };
    }

    if let Some(latency) = probe_tcp(&server.host, agent_port).await {
        let status = if latency > 300 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Online
        };
        return ServerStatusEvent {
            host: server.host.clone(),
            status,
            latency_ms: Some(latency),
        };
    }

    ServerStatusEvent {
        host: server.host.clone(),
        status: HealthStatus::Offline,
        latency_ms: None,
    }
}

pub fn start_ping_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let servers = load_servers(app.clone()).unwrap_or_default();

            let mut tasks = Vec::new();
            for server in servers {
                tasks.push(tokio::spawn(async move { check_server(&server).await }));
            }

            for task in tasks {
                if let Ok(status_event) = task.await {
                    let _ = app.emit("server-status-tick", status_event);
                }
            }

            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    });
}
