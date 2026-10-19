use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub port: u16,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub host: String,
    pub label: String,
    pub description: Option<String>,
    pub agent: Option<Agent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerMonitorSettings {
    pub servers: Vec<Server>,
}
