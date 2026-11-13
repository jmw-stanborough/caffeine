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

pub fn parse_ping_rtt_ms(stdout: &str) -> Option<u64> {
    for line in stdout.lines() {
        for token in line.split_whitespace() {
            let lower = token.to_ascii_lowercase();
            let raw_num = lower
                .strip_prefix("time=")
                .or_else(|| lower.strip_prefix("time<"));

            if let Some(rest) = raw_num {
                let num_part: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if let Ok(val) = num_part.parse::<f64>() {
                    if val.is_finite() && val >= 0.0 {
                        return Some(val.round() as u64);
                    }
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
pub async fn unprivileged_icmp_ping(host: &str) -> Option<u64> {
    let clean_host = strip_host_port(host).trim().to_string();
    if clean_host.is_empty() || clean_host.starts_with('-') {
        return None;
    }

    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "linux")]
        let args = ["-n", "-c", "1", "-W", "1", clean_host.as_str()];
        #[cfg(target_os = "macos")]
        let args = ["-n", "-c", "1", "-t", "1", clean_host.as_str()];
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        let args = ["-n", "-c", "1", clean_host.as_str()];

        let output = crate::process::new_hidden_command("ping")
            .args(args)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_ping_rtt_ms(&stdout)
    })
    .await
    .ok()?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping_rtt_ms_linux_output() {
        let output = "\
PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=14.2 ms

--- 1.1.1.1 ping statistics ---
1 packets transmitted, 1 received, 0% packet loss, time 0ms
rtt min/avg/max/mdev = 14.231/14.231/14.231/0.000 ms";
        assert_eq!(parse_ping_rtt_ms(output), Some(14));
    }

    #[test]
    fn test_parse_ping_rtt_ms_macos_output() {
        let output = "\
PING 192.168.1.1 (192.168.1.1): 56 data bytes
64 bytes from 192.168.1.1: icmp_seq=0 ttl=64 time=0.45 ms

--- 192.168.1.1 ping statistics ---
1 packets transmitted, 1 packets received, 0.0% packet loss
round-trip min/avg/max/stddev = 0.450/0.450/0.450/0.000 ms";
        assert_eq!(parse_ping_rtt_ms(output), Some(0));
    }

    #[test]
    fn test_parse_ping_rtt_ms_sub_millisecond_and_missing() {
        assert_eq!(
            parse_ping_rtt_ms("Reply from 127.0.0.1: bytes=32 time<1ms TTL=128"),
            Some(1)
        );
        assert_eq!(
            parse_ping_rtt_ms("64 bytes from 10.0.0.1: icmp_seq=1 ttl=64 time=19.7 ms"),
            Some(20)
        );
        assert_eq!(
            parse_ping_rtt_ms("1 packets transmitted, 0 received, 100% packet loss"),
            None
        );
        assert_eq!(parse_ping_rtt_ms(""), None);
    }
}
