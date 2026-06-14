use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

#[derive(Clone)]
pub(crate) struct AgentCommandHandle {
    pub(crate) id: u64,
    pub(crate) tx: tokio::sync::mpsc::UnboundedSender<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct App {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) icon: String,
    pub(crate) description: String,
    pub(crate) category: String,
    pub(crate) node_tag: String,
    #[serde(default)]
    pub(crate) mac_address: String,
    #[serde(default)]
    pub(crate) integration_type: String,
    #[serde(default)]
    pub(crate) api_key: String,
}

#[derive(Clone, Serialize)]
pub struct Session {
    pub(crate) username: String,
    pub(crate) role: String,
    pub(crate) expires_at_epoch: u64,
    pub(crate) csrf_token: String,
}

pub(crate) use amud_protocol::{AgentTelemetry, LxcContainer, NetworkTelemetry};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) smart_home: Option<crate::smart_home::SmartHomeTelemetry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PveTestResult {
    pub(crate) success: bool,
    pub(crate) error: Option<String>,
}

#[derive(Clone)]
pub struct ActionResult {
    pub(crate) success: bool,
    pub(crate) error: Option<String>,
    pub(crate) at: Instant,
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

#[allow(dead_code)]
pub struct AppState {
    pub(crate) db: Arc<Mutex<Connection>>,
    pub(crate) sessions: Arc<RwLock<HashMap<String, Session>>>,
    pub(crate) latest_telemetry: Arc<RwLock<AgentTelemetry>>,
    pub(crate) agent_connected: Arc<RwLock<bool>>,
    pub(crate) media_streams: Arc<RwLock<HashMap<String, MediaStream>>>,
    pub(crate) app_statuses: Arc<RwLock<HashMap<String, AppStatus>>>,
    pub(crate) agent_command_tx: Arc<Mutex<Option<AgentCommandHandle>>>,
    pub(crate) next_agent_conn_id: Arc<AtomicU64>,
    pub(crate) pve_test_response: Arc<RwLock<Option<PveTestResult>>>,
    pub(crate) action_results: Arc<RwLock<HashMap<String, ActionResult>>>,
    pub(crate) settings_cache: Arc<RwLock<HashMap<String, String>>>,
    pub(crate) alert_cooldowns: Arc<Mutex<HashMap<String, std::time::Instant>>>,
    pub(crate) login_attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    pub(crate) api_rate_limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    pub(crate) agent_secret: Arc<String>,
    pub(crate) smart_home_telemetry: Arc<RwLock<crate::smart_home::SmartHomeTelemetry>>,
    pub(crate) logo_manifest: Arc<HashMap<String, String>>,
    pub(crate) telemetry_broadcast:
        tokio::sync::watch::Sender<Arc<crate::telemetry_broadcast::WsTelemetryBundle>>,
}
