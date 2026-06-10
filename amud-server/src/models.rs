use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Serialize, Deserialize)]
pub struct App {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) icon: String,
    pub(crate) description: String,
    pub(crate) category: String,
    pub(crate) node_tag: String,
}

#[derive(Clone, Serialize)]
pub struct Session {
    pub(crate) username: String,
    pub(crate) role: String,
    pub(crate) expires_at_epoch: u64,
    pub(crate) csrf_token: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct LxcContainer {
    pub(crate) vmid: i64,
    pub(crate) status: String,
    pub(crate) name: String,
    pub(crate) cpu: Option<f64>,
    pub(crate) maxmem: Option<i64>,
    pub(crate) mem: Option<i64>,
    pub(crate) maxdisk: Option<i64>,
    pub(crate) disk: Option<i64>,
    pub(crate) uptime: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct AgentTelemetry {
    pub(crate) cpu_usage: i32,
    pub(crate) ram_usage: i32,
    pub(crate) ram_used_gb: f64,
    pub(crate) ram_total_gb: f64,
    pub(crate) cpu_temp: f64,
    pub(crate) disk_usage: i32,
    pub(crate) disk_used_gb: f64,
    pub(crate) disk_total_gb: f64,
    #[serde(default)]
    pub(crate) lxc_containers: Vec<LxcContainer>,
    #[serde(default)]
    pub(crate) network: Option<NetworkTelemetry>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct NetworkTelemetry {
    pub(crate) internal_tx: String,
    pub(crate) internal_rx: String,
    pub(crate) external_tx: String,
    pub(crate) external_rx: String,
}

#[derive(Serialize, Clone)]
pub struct MediaStream {
    pub(crate) status: String,
    pub(crate) active: bool,
    pub(crate) title: String,
    pub(crate) current_time: String,
    pub(crate) total_time: String,
    pub(crate) progress_percent: f64,
}

#[derive(Serialize, Clone, Default)]
pub struct AppStatus {
    pub(crate) status: String,
    pub(crate) latency_ms: Option<u128>,
}

#[derive(Serialize, Clone)]
pub struct FullTelemetry {
    pub(crate) system: AgentTelemetry,
    pub(crate) network: NetworkTelemetry,
    pub(crate) streams: HashMap<String, MediaStream>,
    pub(crate) app_statuses: HashMap<String, AppStatus>,
    pub(crate) agent_connected: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PveTestResult {
    pub(crate) success: bool,
    pub(crate) error: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub(crate) success: bool,
    pub(crate) error: Option<String>,
}

#[derive(Deserialize)]
pub struct ActionResultMsg {
    pub(crate) action_result: ActionResultPayload,
}

#[derive(Deserialize)]
pub struct ActionResultPayload {
    pub(crate) request_id: String,
    pub(crate) success: bool,
    pub(crate) error: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) event_types: String,
    pub(crate) is_active: i32,
}

// Global App State
#[allow(dead_code)]
pub struct AppState {
    pub(crate) db: Arc<Mutex<Connection>>,
    pub(crate) sessions: Arc<RwLock<HashMap<String, Session>>>,
    pub(crate) latest_telemetry: Arc<RwLock<AgentTelemetry>>,
    pub(crate) agent_connected: Arc<RwLock<bool>>,
    pub(crate) media_streams: Arc<RwLock<HashMap<String, MediaStream>>>,
    pub(crate) app_statuses: Arc<RwLock<HashMap<String, AppStatus>>>,
    pub(crate) agent_command_tx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<String>>>>,
    pub(crate) pve_test_response: Arc<RwLock<Option<PveTestResult>>>,
    pub(crate) action_results: Arc<RwLock<HashMap<String, ActionResult>>>,
    pub(crate) settings_cache: Arc<RwLock<HashMap<String, String>>>,
    pub(crate) alert_cooldowns: Arc<Mutex<HashMap<String, std::time::Instant>>>,
    pub(crate) login_attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    pub(crate) agent_secret: Arc<String>,
}


