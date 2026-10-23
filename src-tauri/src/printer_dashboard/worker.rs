use super::db::{
    PRINTER_ID_COLUMN, PRINTER_LOGS_TABLE, add_printer_log_entry, get_printers,
    update_printer_detected_location, update_printer_detected_model,
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
    active_empty_toners: HashSet<String>,
    active_paper_jam: bool,
    active_out_of_paper: bool,
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
        if !new_details.toners.is_empty() {
            if self.cached_details.toners.is_empty()
                || new_details.toners.len() >= self.cached_details.toners.len()
            {
                self.cached_details.toners = new_details.toners;
            } else {
                for new_toner in new_details.toners {
                    if let Some(existing) = self.cached_details.toners.iter_mut().find(|t| {
                        (!t.color.is_empty() && t.color.eq_ignore_ascii_case(&new_toner.color))
                            || t.name.eq_ignore_ascii_case(&new_toner.name)
                    }) {
                        *existing = new_toner;
                    }
                }
            }
        }

        let poll_succeeded = new_details.detected_make.is_some()
            || new_details.detected_model.is_some()
            || new_details.detected_location.is_some()
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
        if new_details.detected_location.is_some() {
            self.cached_details.detected_location = new_details.detected_location;
        }
        if new_details.page_count.is_some() {
            self.cached_details.page_count = new_details.page_count;
        }
        if new_details.printer_state.is_some() {
            self.cached_details.printer_state = new_details.printer_state;
        }
        if new_details.is_color.is_some() {
            self.cached_details.is_color = new_details.is_color;
        }
        if new_details.is_duplex.is_some() {
            self.cached_details.is_duplex = new_details.is_duplex;
        }
        if poll_succeeded {
            self.cached_details.status_message = new_details.status_message;
            self.cached_details.paper_jam = new_details.paper_jam;
            self.cached_details.out_of_paper = new_details.out_of_paper;
        }
        self.last_snmp_poll = Some(Instant::now());
    }
}

fn get_empty_toner_keys(details: &PrinterPollDetails) -> HashSet<String> {
    details
        .toners
        .iter()
        .filter(|t| {
            !matches!(t.supply_type.as_str(), "waste_toner" | "waste_ink")
                && (t.percent == Some(0) || (t.percent.is_none() && t.level == 0))
        })
        .map(|t| {
            let color = t.color.trim().to_lowercase();
            if !color.is_empty() {
                color
            } else {
                t.name.trim().to_lowercase()
            }
        })
        .collect()
}

fn has_empty_toner(details: &PrinterPollDetails) -> bool {
    details.toners.iter().any(|t| {
        !matches!(t.supply_type.as_str(), "waste_toner" | "waste_ink")
            && (t.percent == Some(0) || (t.percent.is_none() && t.level == 0))
    })
}

fn update_alert_transitions(
    conn: &Connection,
    printer_id: &str,
    latency_ms: Option<u64>,
    tracker: &mut PrinterTracker,
) {
    let current = &tracker.cached_details;

    if current.paper_jam {
        if !tracker.active_paper_jam {
            tracker.active_paper_jam = true;
            if let Err(e) = add_printer_log_entry(conn, printer_id, EVENT_PAPER_JAM, latency_ms) {
                eprintln!("Failed to record paper jam event for printer {printer_id}: {e}");
            }
        }
    } else {
        tracker.active_paper_jam = false;
    }

    if current.out_of_paper {
        if !tracker.active_out_of_paper {
            tracker.active_out_of_paper = true;
            if let Err(e) = add_printer_log_entry(conn, printer_id, EVENT_OUT_OF_PAPER, latency_ms) {
                eprintln!("Failed to record out of paper event for printer {printer_id}: {e}");
            }
        }
    } else {
        tracker.active_out_of_paper = false;
    }

    let current_empty = get_empty_toner_keys(current);
    let has_newly_empty = current_empty
        .iter()
        .any(|k| !tracker.active_empty_toners.contains(k));
    if has_newly_empty {
        if let Err(e) = add_printer_log_entry(conn, printer_id, EVENT_TONER_EMPTY, latency_ms) {
            eprintln!("Failed to record toner empty event for printer {printer_id}: {e}");
        }
    }
    tracker.active_empty_toners = current_empty;
}

pub fn start_printer_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut trackers = HashMap::<String, PrinterTracker>::new();
        tokio::time::sleep(INITIAL_WORKER_DELAY).await;

        loop {
            let printers = {
                let db = app.state::<DbState>();
                let result = {
                    let conn = match db.0.lock() {
                        Ok(c) => c,
                        Err(e) => panic!("Database lock poisoned in printer worker: {e}"),
                    };
                    get_printers(&conn)
                };
                match result {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Failed to load printers in worker: {e}");
                        tokio::time::sleep(WORKER_POLL_INTERVAL).await;
                        continue;
                    }
                }
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

                if let Some(ref details) = maybe_details {
                    if let Some(ref detected) = details.detected_model {
                        match db.0.lock() {
                            Ok(conn) => {
                                if let Err(e) = update_printer_detected_model(&conn, &printer_id, detected) {
                                    eprintln!("Failed to update detected model for printer {printer_id}: {e}");
                                }
                            }
                            Err(e) => eprintln!("Database lock error updating detected model: {e}"),
                        }
                    }
                    if let Some(ref detected_loc) = details.detected_location {
                        match db.0.lock() {
                            Ok(conn) => {
                                if let Err(e) = update_printer_detected_location(&conn, &printer_id, detected_loc) {
                                    eprintln!("Failed to update detected location for printer {printer_id}: {e}");
                                }
                            }
                            Err(e) => eprintln!("Database lock error updating detected location: {e}"),
                        }
                    }
                }

                let current = if let Some(existing) = trackers.get_mut(&printer_id) {
                    let is_connected = status != HealthStatus::Offline;

                    match record_status_transition(
                        &db,
                        PRINTER_LOGS_TABLE,
                        PRINTER_ID_COLUMN,
                        &printer_id,
                        &existing.status,
                        &status,
                        latency_ms,
                    ) {
                        Ok(Some((ev, ts))) => {
                            existing.last_event = ev;
                            existing.last_event_at = ts;
                        }
                        Ok(None) => {}
                        Err(err) => eprintln!("Failed to record status transition for printer {printer_id}: {err}"),
                    }

                    existing.status = status.clone();
                    if let Some(new_details) = maybe_details {
                        existing.merge_poll_details(new_details);

                        if is_connected {
                            match db.0.lock() {
                                Ok(conn) => {
                                    update_alert_transitions(
                                        &conn,
                                        &printer_id,
                                        latency_ms,
                                        existing,
                                    );
                                }
                                Err(e) => eprintln!("Database lock error updating alert transitions: {e}"),
                            }
                        }
                    }
                    existing
                } else {
                    let event_name = event_name_for_status(&status);
                    let (last_event, last_event_at) = match resolve_initial_event(
                        &db,
                        PRINTER_LOGS_TABLE,
                        PRINTER_ID_COLUMN,
                        &printer_id,
                        event_name,
                        latency_ms,
                    ) {
                        Ok(pair) => pair,
                        Err(err) => {
                            eprintln!("Failed to resolve initial event for printer {printer_id}: {err}");
                            (event_name.to_string(), String::new())
                        }
                    };

                    let poll_time = maybe_details.as_ref().map(|_| Instant::now());
                    let initial_details = maybe_details.unwrap_or_default();
                    let active_empty_toners = get_empty_toner_keys(&initial_details);
                    let active_paper_jam = initial_details.paper_jam;
                    let active_out_of_paper = initial_details.out_of_paper;

                    trackers.entry(printer_id.clone()).or_insert(PrinterTracker {
                        status: status.clone(),
                        last_event,
                        last_event_at,
                        cached_details: initial_details,
                        last_snmp_poll: poll_time,
                        active_empty_toners,
                        active_paper_jam,
                        active_out_of_paper,
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
                    detected_location: current.cached_details.detected_location.clone(),
                    serial_number: current.cached_details.serial_number.clone(),
                    page_count: current.cached_details.page_count,
                    printer_state: current.cached_details.printer_state.clone(),
                    status_message: current.cached_details.status_message.clone(),
                    paper_jam: current.cached_details.paper_jam,
                    out_of_paper: current.cached_details.out_of_paper,
                    is_color: current.cached_details.is_color,
                    is_duplex: current.cached_details.is_duplex,
                };

                if let Err(err) = app.emit(PRINTER_STATUS_EVENT_NAME, event) {
                    eprintln!("Failed to emit printer status event: {err}");
                }
            }

            tokio::time::sleep(WORKER_POLL_INTERVAL).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printer_dashboard::types::TonerSupply;

    #[test]
    fn test_get_empty_toner_keys() {
        let details = PrinterPollDetails {
            toners: vec![
                TonerSupply {
                    name: "Black Cartridge".to_string(),
                    color: "black".to_string(),
                    level: 11,
                    max_capacity: 100,
                    percent: Some(11),
                    supply_type: "toner".to_string(),
                },
                TonerSupply {
                    name: "Yellow Cartridge".to_string(),
                    color: "yellow".to_string(),
                    level: 0,
                    max_capacity: 100,
                    percent: Some(0),
                    supply_type: "toner".to_string(),
                },
                TonerSupply {
                    name: "Waste Toner Box".to_string(),
                    color: "waste".to_string(),
                    level: 0,
                    max_capacity: 100,
                    percent: Some(0),
                    supply_type: "waste_toner".to_string(),
                },
            ],
            ..Default::default()
        };

        let empty = get_empty_toner_keys(&details);
        assert_eq!(empty.len(), 1);
        assert!(empty.contains("yellow"));
    }

    #[test]
    fn test_merge_poll_details_preserves_cartridges_on_partial_walk() {
        let mut tracker = PrinterTracker {
            status: HealthStatus::Online,
            last_event: "connected".to_string(),
            last_event_at: "2026-10-23T08:00:00Z".to_string(),
            cached_details: PrinterPollDetails {
                toners: vec![
                    TonerSupply {
                        name: "Black Cartridge".to_string(),
                        color: "black".to_string(),
                        level: 50,
                        max_capacity: 100,
                        percent: Some(50),
                        supply_type: "toner".to_string(),
                    },
                    TonerSupply {
                        name: "Yellow Cartridge".to_string(),
                        color: "yellow".to_string(),
                        level: 0,
                        max_capacity: 100,
                        percent: Some(0),
                        supply_type: "toner".to_string(),
                    },
                ],
                ..Default::default()
            },
            last_snmp_poll: None,
            active_empty_toners: HashSet::new(),
            active_paper_jam: false,
            active_out_of_paper: false,
        };

        // Partial poll that only received black toner
        let partial = PrinterPollDetails {
            toners: vec![TonerSupply {
                name: "Black Cartridge".to_string(),
                color: "black".to_string(),
                level: 48,
                max_capacity: 100,
                percent: Some(48),
                supply_type: "toner".to_string(),
            }],
            ..Default::default()
        };

        tracker.merge_poll_details(partial);
        assert_eq!(tracker.cached_details.toners.len(), 2);
        assert_eq!(tracker.cached_details.toners[0].percent, Some(48));
        assert_eq!(tracker.cached_details.toners[1].percent, Some(0));
    }

    #[test]
    fn test_update_alert_transitions_deduplicates_toner_empty() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE printer_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                printer_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                latency_ms INTEGER,
                timestamp TEXT NOT NULL DEFAULT '2026-10-23T08:00:00Z'
            );
            ",
        )
        .unwrap();

        let mut tracker = PrinterTracker {
            status: HealthStatus::Online,
            last_event: "connected".to_string(),
            last_event_at: "2026-10-23T08:00:00Z".to_string(),
            cached_details: PrinterPollDetails {
                toners: vec![TonerSupply {
                    name: "Yellow Cartridge".to_string(),
                    color: "yellow".to_string(),
                    level: 0,
                    max_capacity: 100,
                    percent: Some(0),
                    supply_type: "toner".to_string(),
                }],
                ..Default::default()
            },
            last_snmp_poll: None,
            active_empty_toners: HashSet::new(),
            active_paper_jam: false,
            active_out_of_paper: false,
        };

        // First transition: yellow is empty
        update_alert_transitions(&conn, "p1", Some(5), &mut tracker);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM printer_logs WHERE event_type = 'toner empty'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Second poll with yellow still empty: must NOT duplicate
        update_alert_transitions(&conn, "p1", Some(5), &mut tracker);
        let count2: i64 = conn
            .query_row("SELECT COUNT(*) FROM printer_logs WHERE event_type = 'toner empty'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count2, 1);

        // A second cartridge (cyan) becomes empty: must record new transition
        tracker.cached_details.toners.push(TonerSupply {
            name: "Cyan Cartridge".to_string(),
            color: "cyan".to_string(),
            level: 0,
            max_capacity: 100,
            percent: Some(0),
            supply_type: "toner".to_string(),
        });
        update_alert_transitions(&conn, "p1", Some(5), &mut tracker);
        let count3: i64 = conn
            .query_row("SELECT COUNT(*) FROM printer_logs WHERE event_type = 'toner empty'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count3, 2);

        // Yellow is replaced (>0%): should be removed from active_empty_toners
        tracker.cached_details.toners[0].percent = Some(100);
        tracker.cached_details.toners[0].level = 100;
        update_alert_transitions(&conn, "p1", Some(5), &mut tracker);
        assert!(!tracker.active_empty_toners.contains("yellow"));
        assert!(tracker.active_empty_toners.contains("cyan"));
    }
}
