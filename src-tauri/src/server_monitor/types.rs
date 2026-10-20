use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub port: u16,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    #[serde(default = "new_id")]
    pub id: String,
    pub host: String,
    pub label: String,
    pub description: Option<String>,
    pub agent: Option<Agent>,
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Online,
    Offline,
    Degraded,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Online => "online",
            HealthStatus::Offline => "offline",
            HealthStatus::Degraded => "degraded",
        }
    }

    pub fn from_str(val: &str) -> Option<Self> {
        match val {
            "online" => Some(HealthStatus::Online),
            "offline" => Some(HealthStatus::Offline),
            "degraded" => Some(HealthStatus::Degraded),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusEvent {
    pub host: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLog {
    pub id: i64,
    pub server_id: String,
    pub event_type: String,
    pub latency_ms: Option<u64>,
    pub timestamp: String,
}
