use super::db::{DbState, add_log_entry, get_latest_log, get_servers};
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

#[derive(Clone)]
struct ServerTracker {
    status: HealthStatus,
    last_event: String,
    last_event_at: String,
}

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

async fn check_server(server: &Server) -> (HealthStatus, Option<u64>) {
    let agent_port = server.agent.as_ref().map(|Agent { port, .. }| *port);

    if let Some(latency) = unprivileged_icmp_ping(&server.host).await {
        let status = if latency > 300 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Online
        };
        return (status, Some(latency));
    }

    if let Some(latency) = probe_tcp(&server.host, agent_port).await {
        let status = if latency > 300 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Online
        };
        return (status, Some(latency));
    }

    (HealthStatus::Offline, None)
}

pub fn start_ping_worker(app: AppHandle) {
    let trackers = Arc::new(AsyncMutex::new(HashMap::<String, ServerTracker>::new()));

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
                    let (status, latency_ms) = check_server(&s).await;
                    (s.id, s.host, status, latency_ms)
                }));
            }

            for task in tasks {
                if let Ok((server_id, host, status, latency_ms)) = task.await {
                    let mut tracker_map = trackers.lock().await;

                    if !tracker_map.contains_key(&server_id) {
                        let db = app.state::<DbState>();
                        let initial_data = if let Ok(conn) = db.0.lock() {
                            get_latest_log(&conn, &server_id).unwrap_or(None)
                        } else {
                            None
                        };

                        if let Some((ev, at)) = initial_data {
                            tracker_map.insert(
                                server_id.clone(),
                                ServerTracker {
                                    status: status.clone(),
                                    last_event: ev,
                                    last_event_at: at,
                                },
                            );
                        } else {
                            let event_name = match status {
                                HealthStatus::Offline => "disconnected",
                                _ => "connected",
                            };

                            let at = {
                                let db = app.state::<DbState>();
                                if let Ok(conn) = db.0.lock() {
                                    add_log_entry(&conn, &server_id, event_name, latency_ms)
                                        .unwrap_or_default()
                                } else {
                                    String::new()
                                }
                            };

                            tracker_map.insert(
                                server_id.clone(),
                                ServerTracker {
                                    status: status.clone(),
                                    last_event: event_name.to_string(),
                                    last_event_at: at,
                                },
                            );
                        }
                    }

                    let current = tracker_map.get_mut(&server_id).unwrap();
                    let was_connected = current.status != HealthStatus::Offline;
                    let is_connected = status != HealthStatus::Offline;

                    if was_connected != is_connected {
                        let new_event = if is_connected {
                            "connected"
                        } else {
                            "disconnected"
                        };

                        let db = app.state::<DbState>();
                        if let Ok(conn) = db.0.lock() {
                            if let Ok(ts) = add_log_entry(&conn, &server_id, new_event, latency_ms)
                            {
                                current.last_event = new_event.to_string();
                                current.last_event_at = ts;
                            }
                        }
                    }

                    current.status = status.clone();

                    let event = ServerStatusEvent {
                        host,
                        server_id,
                        status,
                        latency_ms,
                        last_event: Some(current.last_event.clone()),
                        last_event_at: Some(current.last_event_at.clone()),
                    };

                    let _ = app.emit("server-status-tick", event);
                }
            }

            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    });
}
