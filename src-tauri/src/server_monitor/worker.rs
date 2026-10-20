use super::db::{DbState, add_log_entry, get_servers};
use super::types::{Agent, HealthStatus, Server, ServerStatusEvent};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpStream;
use tokio::sync::Mutex as AsyncMutex;
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
    let last_statuses = Arc::new(AsyncMutex::new(HashMap::<String, HealthStatus>::new()));

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;

        loop {
            let servers = {
                let db = app.state::<DbState>();
                if let Ok(conn) = db.0.lock() {
                    get_servers(&conn).unwrap_or_default()
                } else {
                    Vec::new()
                }
            };

            let mut tasks = Vec::new();
            for server in servers {
                let s = server.clone();
                tasks.push(tokio::spawn(async move {
                    let event = check_server(&s).await;
                    (s.id, event)
                }));
            }

            for task in tasks {
                if let Ok((server_id, status_event)) = task.await {
                    let mut history = last_statuses.lock().await;
                    let prev = history.get(&server_id).cloned();

                    if let Some(ref old_status) = prev {
                        if old_status != &status_event.status {
                            let db = app.state::<DbState>();
                            if let Ok(conn) = db.0.lock() {
                                let _ = add_log_entry(
                                    &conn,
                                    &server_id,
                                    status_event.status.as_str(),
                                    status_event.latency_ms,
                                );
                            }
                        }
                    }

                    history.insert(server_id, status_event.status.clone());
                    let _ = app.emit("server-status-tick", status_event);
                }
            }

            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    });
}
