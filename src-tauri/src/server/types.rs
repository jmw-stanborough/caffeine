use crate::printer_dashboard::types::{Printer, PrinterFilterConfig, PrinterLog, PrinterStatusEvent};
use crate::server_monitor::types::{Server, ServerLog, ServerStatusEvent};
use serde::{Deserialize, Serialize};

pub const REDACTED_SECRET: &str = "__REDACTED__";

pub fn sanitize_secret_update(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == REDACTED_SECRET {
        ""
    } else {
        trimmed
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_secret_update() {
        assert_eq!(sanitize_secret_update(""), "");
        assert_eq!(sanitize_secret_update("   "), "");
        assert_eq!(sanitize_secret_update(REDACTED_SECRET), "");
        assert_eq!(sanitize_secret_update(&format!("  {REDACTED_SECRET}  ")), "");
        assert_eq!(sanitize_secret_update("  my_secret  "), "my_secret");
    }
}
