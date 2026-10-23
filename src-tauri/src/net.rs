use crate::server_monitor::types::HealthStatus;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(1000);
pub const DEGRADED_LATENCY_MS: u64 = 300;
pub const WORKER_POLL_INTERVAL: Duration = Duration::from_secs(4);

pub fn strip_host_port(host: &str) -> &str {
    host.split(':').next().unwrap_or(host)
}

#[cfg(target_os = "windows")]
pub async fn unprivileged_icmp_ping(host: &str) -> Option<u64> {
    let clean_host = strip_host_port(host).to_string();
    tokio::task::spawn_blocking(move || {
        use std::net::ToSocketAddrs;
        use winping::{Buffer, Pinger};

        let ip = if let Ok(ip) = clean_host.parse::<std::net::IpAddr>() {
            ip
        } else {
            format!("{clean_host}:0")
                .to_socket_addrs()
                .ok()?
                .next()?
                .ip()
        };

        let mut pinger = Pinger::new().ok()?;
        pinger.set_timeout(1000);
        let mut buffer = Buffer::new();

        for _ in 0..2 {
            if let Ok(rtt) = pinger.send(ip, &mut buffer) {
                return Some(rtt as u64);
            }
        }
        None
    })
    .await
    .ok()?
}

#[cfg(not(target_os = "windows"))]
pub async fn unprivileged_icmp_ping(_host: &str) -> Option<u64> {
    None
}

/// Probes a list of TCP ports sequentially as a fallback when ICMP is blocked or unavailable.
/// A `ConnectionRefused` response still proves the host's network stack is reachable.
pub async fn probe_tcp_ports(host: &str, ports: &[u16]) -> Option<u64> {
    let clean_host = strip_host_port(host);
    for &port in ports {
        let start = Instant::now();
        let target = format!("{clean_host}:{port}");

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

pub async fn check_host_reachability(
    host: &str,
    fallback_ports: &[u16],
) -> (HealthStatus, Option<u64>) {
    let latency = match unprivileged_icmp_ping(host).await {
        Some(ms) => Some(ms),
        None => probe_tcp_ports(host, fallback_ports).await,
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

pub fn event_name_for_status(status: &HealthStatus) -> &'static str {
    match status {
        HealthStatus::Offline => "disconnected",
        HealthStatus::Online | HealthStatus::Degraded => "connected",
    }
}
