use crate::server_monitor::types::HealthStatus;
use serde::{Deserialize, Serialize};

pub const DEFAULT_SNMP_COMMUNITY: &str = "public";

pub fn normalize_snmp_community(raw: &str) -> &str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        DEFAULT_SNMP_COMMUNITY
    } else {
        trimmed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Printer {
    #[serde(default = "new_id")]
    pub id: String,
    pub host: String,
    pub label: String,
    pub description: Option<String>,
    pub model: Option<String>,
    pub location: Option<String>,
    #[serde(default = "default_community")]
    pub snmp_community: String,
    pub web_url: Option<String>,
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_community() -> String {
    DEFAULT_SNMP_COMMUNITY.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonerSupply {
    pub name: String,
    pub color: String,
    pub level: i32,
    pub max_capacity: i32,
    pub percent: Option<u8>,
    pub supply_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterStatusEvent {
    pub host: String,
    pub printer_id: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub last_event: Option<String>,
    pub last_event_at: Option<String>,
    pub toners: Vec<TonerSupply>,
    pub detected_make: Option<String>,
    pub detected_model: Option<String>,
    pub serial_number: Option<String>,
    pub page_count: Option<u64>,
    pub printer_state: Option<String>,
    pub status_message: Option<String>,
    pub paper_jam: bool,
    pub out_of_paper: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterLog {
    pub id: i64,
    pub printer_id: String,
    pub event_type: String,
    pub latency_ms: Option<u64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrinterFilterConfig {
    #[serde(default)]
    pub all_tags: Vec<String>,
    #[serde(default)]
    pub any_tags: Vec<String>,
}
