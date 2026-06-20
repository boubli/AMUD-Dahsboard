use crate::models::{AppState, AppStatus, FullTelemetry};
use amud_protocol::{AgentTelemetry, NetworkTelemetry};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::watch;

#[derive(Clone)]
pub struct WsTelemetryBundle {
    pub full: Arc<str>,
    pub guest_public: Arc<str>,
    pub guest_redacted: Arc<str>,
}

fn empty_full_telemetry() -> FullTelemetry {
    FullTelemetry {
        system: AgentTelemetry::default(),
        network: NetworkTelemetry::default(),
        streams: HashMap::new(),
        app_statuses: HashMap::new(),
        agent_connected: false,
        smart_home: None,
    }
}

fn read_rwlock<T: Clone>(lock: &RwLock<T>) -> T {
    lock.read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

fn read_agent_connected(lock: &RwLock<bool>) -> bool {
    *lock.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Default for WsTelemetryBundle {
    fn default() -> Self {
        let empty = empty_full_telemetry();
        Self::from_payloads(&empty, &empty, &empty)
    }
}

impl WsTelemetryBundle {
    fn from_payloads(
        full: &FullTelemetry,
        guest_public: &FullTelemetry,
        guest_redacted: &FullTelemetry,
    ) -> Self {
        fn encode(payload: &FullTelemetry) -> Arc<str> {
            Arc::from(serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string()))
        }
        Self {
            full: encode(full),
            guest_public: encode(guest_public),
            guest_redacted: encode(guest_redacted),
        }
    }

    pub(crate) fn from_state(state: &AppState) -> Self {
        let system = read_rwlock(&state.latest_telemetry);
        let streams = read_rwlock(&state.media_streams);
        let app_statuses = read_rwlock(&state.app_statuses);
        let agent_connected = read_agent_connected(&state.agent_connected);
        let smart_home = read_rwlock(&state.smart_home_telemetry);

        let network = system.network.clone().unwrap_or_default();

        let guest_app_statuses: HashMap<String, AppStatus> = app_statuses
            .iter()
            .map(|(name, status)| {
                let public_status = if status.status.eq_ignore_ascii_case("ONLINE") {
                    "ONLINE"
                } else {
                    "OFFLINE"
                };
                (
                    name.clone(),
                    AppStatus {
                        status: public_status.to_string(),
                        latency_ms: None,
                    },
                )
            })
            .collect();

        let mut guest_system = system.clone();
        guest_system.lxc_containers.clear();

        let full = FullTelemetry {
            system,
            network: network.clone(),
            streams,
            app_statuses,
            agent_connected,
            smart_home: Some(smart_home),
        };

        let guest_public = FullTelemetry {
            system: guest_system,
            network,
            streams: HashMap::new(),
            app_statuses: guest_app_statuses.clone(),
            agent_connected: false,
            smart_home: None,
        };

        let guest_redacted = FullTelemetry {
            system: AgentTelemetry::default(),
            network: NetworkTelemetry::default(),
            streams: HashMap::new(),
            app_statuses: guest_app_statuses,
            agent_connected: false,
            smart_home: None,
        };

        Self::from_payloads(&full, &guest_public, &guest_redacted)
    }
}

pub(crate) fn new_telemetry_broadcast() -> watch::Sender<Arc<WsTelemetryBundle>> {
    let (tx, _) = watch::channel(Arc::new(WsTelemetryBundle::default()));
    tx
}

pub(crate) fn start_telemetry_broadcaster(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            let bundle = Arc::new(WsTelemetryBundle::from_state(&state));
            let _ = state.telemetry_broadcast.send(bundle);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AppState;
    use amud_protocol::LxcContainer;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicU64;
    use std::sync::{Mutex, RwLock};

    fn test_state(app_statuses: HashMap<String, AppStatus>) -> AppState {
        let system = AgentTelemetry {
            cpu_usage: 42,
            lxc_containers: vec![LxcContainer {
                vmid: 200,
                status: "running".to_string(),
                name: "jellyfin".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        AppState {
            db: Arc::new(Mutex::new(
                rusqlite::Connection::open_in_memory().expect("in-memory db"),
            )),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            latest_telemetry: Arc::new(RwLock::new(system)),
            agent_connected: Arc::new(RwLock::new(true)),
            media_streams: Arc::new(RwLock::new(HashMap::new())),
            app_statuses: Arc::new(RwLock::new(app_statuses)),
            agent_command_tx: Arc::new(Mutex::new(None)),
            next_agent_conn_id: Arc::new(AtomicU64::new(1)),
            pve_test_response: Arc::new(RwLock::new(None)),
            action_results: Arc::new(RwLock::new(HashMap::new())),
            settings_cache: Arc::new(RwLock::new(HashMap::new())),
            alert_cooldowns: Arc::new(Mutex::new(HashMap::new())),
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            api_rate_limits: Arc::new(Mutex::new(HashMap::new())),
            agent_secret: Arc::new("test-secret".to_string()),
            smart_home_telemetry: Arc::new(RwLock::new(Default::default())),
            logo_manifest: Arc::new(HashMap::new()),
            telemetry_broadcast: new_telemetry_broadcast(),
        }
    }

    #[test]
    fn guest_redacted_includes_online_offline_only() {
        let mut statuses = HashMap::new();
        statuses.insert(
            "jellyfin".to_string(),
            AppStatus {
                status: "ONLINE".to_string(),
                latency_ms: Some(12),
            },
        );
        statuses.insert(
            "radarr".to_string(),
            AppStatus {
                status: "BLOCKED".to_string(),
                latency_ms: None,
            },
        );

        let bundle = WsTelemetryBundle::from_state(&test_state(statuses));
        let guest: serde_json::Value =
            serde_json::from_str(&bundle.guest_redacted).expect("telemetry json");

        assert!(guest["system"]["lxc_containers"]
            .as_array()
            .is_some_and(|v| v.is_empty()));
        assert_eq!(guest["system"]["cpu_usage"], 0);
        assert_eq!(guest["agent_connected"], false);
        assert!(guest.get("smart_home").is_none());

        assert_eq!(guest["app_statuses"]["jellyfin"]["status"], "ONLINE");
        assert!(guest["app_statuses"]["jellyfin"]["latency_ms"].is_null());

        assert_eq!(guest["app_statuses"]["radarr"]["status"], "OFFLINE");
    }

    #[test]
    fn guest_public_hides_lxc_but_keeps_basic_system_metrics() {
        let bundle = WsTelemetryBundle::from_state(&test_state(HashMap::new()));
        let guest: serde_json::Value =
            serde_json::from_str(&bundle.guest_public).expect("telemetry json");

        assert!(guest["system"]["lxc_containers"]
            .as_array()
            .is_some_and(|v| v.is_empty()));
        assert_eq!(guest["system"]["cpu_usage"], 42);
    }
}
