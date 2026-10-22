use super::db::{DbState, add_log_entry, get_latest_log, get_servers};
use super::types::{Agent, HealthStatus, Server, ServerStatusEvent};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpStream;
use tokio::time::timeout;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(1000);
const DEGRADED_LATENCY_MS: u64 = 300;
const PROBE_PORTS: &[u16] = &[445, 135, 80, 443, 53, 22, 3389];

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
    let mut ports = Vec::with_capacity(PROBE_PORTS.len() + usize::from(custom_port.is_some()));
    if let Some(port) = custom_port {
        ports.push(port);
    }
    ports.extend_from_slice(PROBE_PORTS);

    for port in ports {
        let start = Instant::now();
        let target = format!("{host}:{port}");

        match timeout(CONNECT_TIMEOUT, TcpStream::connect(&target)).await {
            Ok(Ok(_)) => return Some(start.elapsed().as_millis() as u64),
            Ok(Err(err)) if err.kind() == std::io::ErrorKind::ConnectionRefused => {
                return Some(start.elapsed().as_millis() as u64);
            }
            _ => continue,
        }
    }

    None
}

async fn check_server(server: &Server) -> (HealthStatus, Option<u64>) {
    let agent_port = server.agent.as_ref().map(|Agent { port, .. }| *port);

    let latency = match unprivileged_icmp_ping(&server.host).await {
        Some(ms) => Some(ms),
        None => probe_tcp(&server.host, agent_port).await,
    };

    match latency {
        Some(ms) => {
            let status = if ms > DEGRADED_LATENCY_MS {
                HealthStatus::Degraded
            } else {
                HealthStatus::Online
            };
            (status, Some(ms))
        }
        None => (HealthStatus::Offline, None),
    }
}

fn event_name_for_status(status: &HealthStatus) -> &'static str {
    match status {
        HealthStatus::Offline => "disconnected",
        HealthStatus::Online | HealthStatus::Degraded => "connected",
    }
}

pub fn start_ping_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut trackers = HashMap::<String, ServerTracker>::new();
        tokio::time::sleep(Duration::from_millis(500)).await;

        loop {
            let servers = {
                let db = app.state::<DbState>();
                db.0.lock()
                    .ok()
                    .and_then(|conn| get_servers(&conn).ok())
                    .unwrap_or_default()
            };

            let active_ids: HashSet<String> = servers.iter().map(|s| s.id.clone()).collect();
            trackers.retain(|id, _| active_ids.contains(id));

            let tasks: Vec<_> = servers
                .into_iter()
                .map(|server| {
                    tokio::spawn(async move {
                        let (status, latency_ms) = check_server(&server).await;
                        (server.id, server.host, status, latency_ms)
                    })
                })
                .collect();

            for task in tasks {
                let Ok((server_id, host, status, latency_ms)) = task.await else {
                    continue;
                };

                let current = if let Some(existing) = trackers.get_mut(&server_id) {
                    let was_connected = existing.status != HealthStatus::Offline;
                    let is_connected = status != HealthStatus::Offline;

                    if was_connected != is_connected {
                        let new_event = event_name_for_status(&status);
                        let db = app.state::<DbState>();
                        if let Ok(conn) = db.0.lock()
                            && let Ok(ts) = add_log_entry(&conn, &server_id, new_event, latency_ms)
                        {
                            existing.last_event = new_event.to_string();
                            existing.last_event_at = ts;
                        }
                    }

                    existing.status = status.clone();
                    existing
                } else {
                    let event_name = event_name_for_status(&status);
                    let (last_event, last_event_at) = {
                        let db = app.state::<DbState>();
                        if let Ok(conn) = db.0.lock() {
                            if let Some((ev, at)) = get_latest_log(&conn, &server_id).unwrap_or(None)
                            {
                                (ev, at)
                            } else {
                                let at = add_log_entry(&conn, &server_id, event_name, latency_ms)
                                    .unwrap_or_default();
                                (event_name.to_string(), at)
                            }
                        } else {
                            (event_name.to_string(), String::new())
                        }
                    };

                    trackers.entry(server_id.clone()).or_insert(ServerTracker {
                        status: status.clone(),
                        last_event,
                        last_event_at,
                    })
                };

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

            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    });
}
