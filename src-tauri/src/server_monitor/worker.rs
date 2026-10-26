use super::db::{SERVER_ID_COLUMN, SERVER_LOGS_TABLE, get_servers};
use super::types::{Agent, HealthStatus, Server, ServerStatusEvent};
use crate::db::{DbState, record_status_transition, resolve_initial_event};
use crate::net::{WORKER_POLL_INTERVAL, check_host_reachability, event_name_for_status};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const INITIAL_WORKER_DELAY: Duration = Duration::from_millis(500);
const DEFAULT_SERVER_PROBE_PORTS: &[u16] = &[445, 135, 80, 443, 53, 22, 3389];
const SERVER_STATUS_EVENT_NAME: &str = "server-status-tick";

struct ServerTracker {
    status: HealthStatus,
    last_event: String,
    last_event_at: String,
}

async fn check_server(server: &Server) -> (HealthStatus, Option<u64>) {
    let custom_port = server.agent.as_ref().map(|Agent { port, .. }| *port);
    let mut ports =
        Vec::with_capacity(DEFAULT_SERVER_PROBE_PORTS.len() + usize::from(custom_port.is_some()));
    if let Some(port) = custom_port {
        ports.push(port);
    }
    ports.extend_from_slice(DEFAULT_SERVER_PROBE_PORTS);

    check_host_reachability(&server.host, &ports).await
}

pub fn start_ping_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut trackers = HashMap::<String, ServerTracker>::new();
        tokio::time::sleep(INITIAL_WORKER_DELAY).await;

        loop {
            let settings = crate::config::load_app_settings(&app).unwrap_or_default();
            if !crate::config::is_self_serving(&settings.target_host) && !settings.server_mode {
                tokio::time::sleep(WORKER_POLL_INTERVAL).await;
                continue;
            }

            let servers = {
                let db = app.state::<DbState>();
                let result = {
                    let conn = match db.0.lock() {
                        Ok(c) => c,
                        Err(e) => panic!("Database lock poisoned in server monitor worker: {e}"),
                    };
                    get_servers(&conn)
                };
                match result {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to load servers in monitor worker: {e}");
                        tokio::time::sleep(WORKER_POLL_INTERVAL).await;
                        continue;
                    }
                }
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

                let db = app.state::<DbState>();
                let current = if let Some(existing) = trackers.get_mut(&server_id) {
                    match record_status_transition(
                        &db,
                        SERVER_LOGS_TABLE,
                        SERVER_ID_COLUMN,
                        &server_id,
                        &existing.status,
                        &status,
                        latency_ms,
                    ) {
                        Ok(Some((ev, ts))) => {
                            existing.last_event = ev;
                            existing.last_event_at = ts;
                        }
                        Ok(None) => {}
                        Err(err) => {
                            eprintln!("Failed to record status transition for server {server_id}: {err}");
                        }
                    }

                    existing.status = status.clone();
                    existing
                } else {
                    let event_name = event_name_for_status(&status);
                    let (last_event, last_event_at) = match resolve_initial_event(
                        &db,
                        SERVER_LOGS_TABLE,
                        SERVER_ID_COLUMN,
                        &server_id,
                        event_name,
                        latency_ms,
                    ) {
                        Ok(pair) => pair,
                        Err(err) => {
                            eprintln!("Failed to resolve initial event for server {server_id}: {err}");
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

                if let Err(err) = app.emit(SERVER_STATUS_EVENT_NAME, &event) {
                    eprintln!("Failed to emit server status event: {err}");
                }

                if let Some(active_srv) = app.try_state::<crate::server::ActiveServer>() {
                    active_srv.broadcast(crate::server::types::ServerBroadcastEvent::ServerTick(event));
                }
            }

            tokio::time::sleep(WORKER_POLL_INTERVAL).await;
        }
    });
}
