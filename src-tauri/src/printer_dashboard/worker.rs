use super::db::{
    PRINTER_ID_COLUMN, PRINTER_LOGS_TABLE, add_printer_log_entry, get_printers,
    update_printer_detected_model,
};
use super::snmp::{PrinterPollDetails, poll_printer_details};
use super::types::PrinterStatusEvent;
use crate::db::{DbState, record_status_transition, resolve_initial_event};
use crate::net::{
    WORKER_POLL_INTERVAL, check_host_reachability, event_name_for_status, strip_host_port,
};
use crate::server_monitor::types::HealthStatus;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

const INITIAL_WORKER_DELAY: Duration = Duration::from_millis(600);
const ALERT_SNMP_POLL_INTERVAL: Duration = Duration::from_secs(4);
const NORMAL_SNMP_POLL_INTERVAL: Duration = Duration::from_secs(12);
const PRINTER_PROBE_PORTS: &[u16] = &[80, 443, 631, 9100, 515, 8080];
const PRINTER_STATUS_EVENT_NAME: &str = "printer-status-tick";

const EVENT_PAPER_JAM: &str = "paper jam";
const EVENT_OUT_OF_PAPER: &str = "out of paper";
const EVENT_TONER_EMPTY: &str = "toner empty";

struct PrinterTracker {
    status: HealthStatus,
    last_event: String,
    last_event_at: String,
    cached_details: PrinterPollDetails,
    last_snmp_poll: Option<Instant>,
}

impl PrinterTracker {
    fn should_poll_snmp(&self) -> bool {
        let interval = if self.cached_details.paper_jam
            || self.cached_details.out_of_paper
            || has_empty_toner(&self.cached_details)
        {
            ALERT_SNMP_POLL_INTERVAL
        } else {
            NORMAL_SNMP_POLL_INTERVAL
        };
        self.last_snmp_poll
            .is_none_or(|last| last.elapsed() >= interval)
    }

    fn merge_poll_details(&mut self, new_details: PrinterPollDetails) {
        if !new_details.toners.is_empty() || self.cached_details.toners.is_empty() {
            self.cached_details = new_details;
        } else {
            // Retain previously polled toners if a transient UDP packet drop prevented the
            // supply table walk from completing, while still updating any fields that succeeded.
            let poll_succeeded = new_details.detected_make.is_some()
                || new_details.detected_model.is_some()
                || new_details.printer_state.is_some()
                || new_details.page_count.is_some()
                || new_details.paper_jam
                || new_details.out_of_paper;

            if new_details.detected_make.is_some() {
                self.cached_details.detected_make = new_details.detected_make;
            }
            if new_details.detected_model.is_some() {
                self.cached_details.detected_model = new_details.detected_model;
            }
            if new_details.page_count.is_some() {
                self.cached_details.page_count = new_details.page_count;
            }
            if new_details.printer_state.is_some() {
                self.cached_details.printer_state = new_details.printer_state;
            }
            if poll_succeeded {
                self.cached_details.status_message = new_details.status_message;
                self.cached_details.paper_jam = new_details.paper_jam;
                self.cached_details.out_of_paper = new_details.out_of_paper;
            }
        }
        self.last_snmp_poll = Some(Instant::now());
    }
}

fn has_empty_toner(details: &PrinterPollDetails) -> bool {
    details.toners.iter().any(|t| {
        !matches!(t.supply_type.as_str(), "waste_toner" | "waste_ink")
            && (t.percent == Some(0) || (t.percent.is_none() && t.level == 0))
    })
}

fn record_alert_transitions(
    conn: &Connection,
    printer_id: &str,
    latency_ms: Option<u64>,
    had_jam: bool,
    had_no_paper: bool,
    had_empty_toner: bool,
    current: &PrinterPollDetails,
) {
    if !had_jam && current.paper_jam {
        let _ = add_printer_log_entry(conn, printer_id, EVENT_PAPER_JAM, latency_ms);
    }
    if !had_no_paper && current.out_of_paper {
        let _ = add_printer_log_entry(conn, printer_id, EVENT_OUT_OF_PAPER, latency_ms);
    }
    if !had_empty_toner && has_empty_toner(current) {
        let _ = add_printer_log_entry(conn, printer_id, EVENT_TONER_EMPTY, latency_ms);
    }
}

pub fn start_printer_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut trackers = HashMap::<String, PrinterTracker>::new();
        tokio::time::sleep(INITIAL_WORKER_DELAY).await;

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
                        .is_none_or(PrinterTracker::should_poll_snmp);

                    tokio::spawn(async move {
                        let (status, latency_ms) =
                            check_host_reachability(&printer.host, PRINTER_PROBE_PORTS).await;
                        let details = if status != HealthStatus::Offline && should_poll_details {
                            let clean_host = strip_host_port(&printer.host);
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

                let db = app.state::<DbState>();

                if let Some(ref details) = maybe_details
                    && let Some(ref detected) = details.detected_model
                    && let Ok(conn) = db.0.lock()
                {
                    let _ = update_printer_detected_model(&conn, &printer_id, detected);
                }

                let current = if let Some(existing) = trackers.get_mut(&printer_id) {
                    let is_connected = status != HealthStatus::Offline;
                    let had_jam = existing.cached_details.paper_jam;
                    let had_no_paper = existing.cached_details.out_of_paper;
                    let had_empty_toner = has_empty_toner(&existing.cached_details);

                    if let Some((ev, ts)) = record_status_transition(
                        &db,
                        PRINTER_LOGS_TABLE,
                        PRINTER_ID_COLUMN,
                        &printer_id,
                        &existing.status,
                        &status,
                        latency_ms,
                    ) {
                        existing.last_event = ev;
                        existing.last_event_at = ts;
                    }

                    existing.status = status.clone();
                    if let Some(new_details) = maybe_details {
                        existing.merge_poll_details(new_details);

                        if is_connected && let Ok(conn) = db.0.lock() {
                            record_alert_transitions(
                                &conn,
                                &printer_id,
                                latency_ms,
                                had_jam,
                                had_no_paper,
                                had_empty_toner,
                                &existing.cached_details,
                            );
                        }
                    }
                    existing
                } else {
                    let event_name = event_name_for_status(&status);
                    let (last_event, last_event_at) = resolve_initial_event(
                        &db,
                        PRINTER_LOGS_TABLE,
                        PRINTER_ID_COLUMN,
                        &printer_id,
                        event_name,
                        latency_ms,
                    );

                    let poll_time = maybe_details.as_ref().map(|_| Instant::now());
                    let initial_details = maybe_details.unwrap_or_default();
                    if status != HealthStatus::Offline
                        && let Ok(conn) = db.0.lock()
                    {
                        record_alert_transitions(
                            &conn,
                            &printer_id,
                            latency_ms,
                            false,
                            false,
                            false,
                            &initial_details,
                        );
                    }

                    trackers.entry(printer_id.clone()).or_insert(PrinterTracker {
                        status: status.clone(),
                        last_event,
                        last_event_at,
                        cached_details: initial_details,
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
                    detected_make: current.cached_details.detected_make.clone(),
                    detected_model: current.cached_details.detected_model.clone(),
                    serial_number: current.cached_details.serial_number.clone(),
                    page_count: current.cached_details.page_count,
                    printer_state: current.cached_details.printer_state.clone(),
                    status_message: current.cached_details.status_message.clone(),
                    paper_jam: current.cached_details.paper_jam,
                    out_of_paper: current.cached_details.out_of_paper,
                };

                let _ = app.emit(PRINTER_STATUS_EVENT_NAME, event);
            }

            tokio::time::sleep(WORKER_POLL_INTERVAL).await;
        }
    });
}
