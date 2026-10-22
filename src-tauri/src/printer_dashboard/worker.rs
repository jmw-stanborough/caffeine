use super::db::{
    add_printer_log_entry, get_latest_printer_log, get_printers, update_printer_detected_model,
};
use super::snmp::{PrinterPollDetails, poll_printer_details};
use super::types::{Printer, PrinterStatusEvent};
use crate::server_monitor::db::DbState;
use crate::server_monitor::types::HealthStatus;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpStream;
use tokio::time::timeout;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(1000);
const DEGRADED_LATENCY_MS: u64 = 300;
const PRINTER_PROBE_PORTS: &[u16] = &[80, 443, 631, 9100, 515, 8080];

struct PrinterTracker {
    status: HealthStatus,
    last_event: String,
    last_event_at: String,
    cached_details: PrinterPollDetails,
    last_snmp_poll: Option<Instant>,
}

#[cfg(target_os = "windows")]
async fn unprivileged_icmp_ping(host: &str) -> Option<u64> {
    let host = host.split(':').next().unwrap_or(host).to_string();
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

async fn probe_printer_tcp(host: &str) -> Option<u64> {
    let clean_host = host.split(':').next().unwrap_or(host);
    for port in PRINTER_PROBE_PORTS {
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

async fn check_printer_reachability(printer: &Printer) -> (HealthStatus, Option<u64>) {
    let latency = match unprivileged_icmp_ping(&printer.host).await {
        Some(ms) => Some(ms),
        None => probe_printer_tcp(&printer.host).await,
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

pub fn start_printer_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut trackers = HashMap::<String, PrinterTracker>::new();
        tokio::time::sleep(Duration::from_millis(600)).await;

        loop {
            let printers = {
                let db = app.state::<DbState>();
                db.0.lock()
                    .ok()
                    .and_then(|conn| get_printers(&conn).ok())
                    .unwrap_or_default()
            };

            let active_ids: HashSet<String> = printers.iter().map(|p| p.id.clone()).collect();
            trackers.retain(|id, _| active_ids.contains(id));

            let tasks: Vec<_> = printers
                .into_iter()
                .map(|printer| {
                    let should_poll_details = trackers
                        .get(&printer.id)
                        .and_then(|t| t.last_snmp_poll)
                        .is_none_or(|last| last.elapsed() >= Duration::from_secs(12));

                    tokio::spawn(async move {
                        let (status, latency_ms) = check_printer_reachability(&printer).await;
                        let details = if status != HealthStatus::Offline && should_poll_details {
                            let clean_host =
                                printer.host.split(':').next().unwrap_or(&printer.host);
                            Some(poll_printer_details(clean_host, &printer.snmp_community).await)
                        } else {
                            None
                        };
                        (printer.id, printer.host, status, latency_ms, details)
                    })
                })
                .collect();

            for task in tasks {
                let Ok((printer_id, host, status, latency_ms, maybe_details)) = task.await else {
                    continue;
                };

                if let Some(ref details) = maybe_details
                    && let Some(ref detected) = details.detected_model
                {
                    let db = app.state::<DbState>();
                    if let Ok(conn) = db.0.lock() {
                        let _ = update_printer_detected_model(&conn, &printer_id, detected);
                    }
                }

                let current = if let Some(existing) = trackers.get_mut(&printer_id) {
                    let was_connected = existing.status != HealthStatus::Offline;
                    let is_connected = status != HealthStatus::Offline;

                    if was_connected != is_connected {
                        let new_event = event_name_for_status(&status);
                        let db = app.state::<DbState>();
                        if let Ok(conn) = db.0.lock()
                            && let Ok(ts) =
                                add_printer_log_entry(&conn, &printer_id, new_event, latency_ms)
                        {
                            existing.last_event = new_event.to_string();
                            existing.last_event_at = ts;
                        }
                    }

                    existing.status = status.clone();
                    if let Some(new_details) = maybe_details {
                        if !new_details.toners.is_empty() || existing.cached_details.toners.is_empty() {
                            existing.cached_details = new_details;
                        } else {
                            // Keep existing toners if a transient UDP drop happened, update other fields
                            if new_details.detected_model.is_some() {
                                existing.cached_details.detected_model = new_details.detected_model;
                            }
                            if new_details.page_count.is_some() {
                                existing.cached_details.page_count = new_details.page_count;
                            }
                            if new_details.printer_state.is_some() {
                                existing.cached_details.printer_state = new_details.printer_state;
                            }
                            if new_details.status_message.is_some() {
                                existing.cached_details.status_message = new_details.status_message;
                            }
                        }
                        existing.last_snmp_poll = Some(Instant::now());
                    }
                    existing
                } else {
                    let event_name = event_name_for_status(&status);
                    let (last_event, last_event_at) = {
                        let db = app.state::<DbState>();
                        if let Ok(conn) = db.0.lock() {
                            if let Some((ev, at)) =
                                get_latest_printer_log(&conn, &printer_id).unwrap_or(None)
                            {
                                (ev, at)
                            } else {
                                let at = add_printer_log_entry(
                                    &conn,
                                    &printer_id,
                                    event_name,
                                    latency_ms,
                                )
                                .unwrap_or_default();
                                (event_name.to_string(), at)
                            }
                        } else {
                            (event_name.to_string(), String::new())
                        }
                    };

                    let poll_time = maybe_details.as_ref().map(|_| Instant::now());
                    trackers.entry(printer_id.clone()).or_insert(PrinterTracker {
                        status: status.clone(),
                        last_event,
                        last_event_at,
                        cached_details: maybe_details.unwrap_or_default(),
                        last_snmp_poll: poll_time,
                    })
                };

                let event = PrinterStatusEvent {
                    host,
                    printer_id,
                    status,
                    latency_ms,
                    last_event: Some(current.last_event.clone()),
                    last_event_at: Some(current.last_event_at.clone()),
                    toners: current.cached_details.toners.clone(),
                    detected_model: current.cached_details.detected_model.clone(),
                    serial_number: current.cached_details.serial_number.clone(),
                    page_count: current.cached_details.page_count,
                    printer_state: current.cached_details.printer_state.clone(),
                    status_message: current.cached_details.status_message.clone(),
                };

                let _ = app.emit("printer-status-tick", event);
            }

            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    });
}
