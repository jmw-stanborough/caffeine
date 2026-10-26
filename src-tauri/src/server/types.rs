use crate::printer_dashboard::types::{Printer, PrinterFilterConfig, PrinterLog, PrinterStatusEvent};
use crate::server_monitor::types::{Server, ServerLog, ServerStatusEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerBroadcastEvent {
    ServerTick(ServerStatusEvent),
    PrinterTick(PrinterStatusEvent),
    EntitiesUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub servers: Vec<Server>,
    pub printers: Vec<Printer>,
    pub printer_filter_tags: PrinterFilterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySyncResponse {
    pub server_logs: Vec<ServerLog>,
    pub printer_logs: Vec<PrinterLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub role: String,
}
