use crate::models::{AppState, FullTelemetry};
use amud_protocol::{AgentTelemetry, NetworkTelemetry};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

#[derive(Clone)]
pub(crate) struct WsTelemetryBundle {
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
        let system = state.latest_telemetry.read().unwrap().clone();
        let streams = state.media_streams.read().unwrap().clone();
        let app_statuses = state.app_statuses.read().unwrap().clone();
        let agent_connected = *state.agent_connected.read().unwrap();
        let smart_home = state.smart_home_telemetry.read().unwrap().clone();

        let full = FullTelemetry {
            system: system.clone(),
            network: system.network.clone().unwrap_or_default(),
            streams,
            app_statuses,
            agent_connected,
            smart_home: Some(smart_home),
        };

        let mut guest_system = system.clone();
        guest_system.lxc_containers.clear();
        let guest_public = FullTelemetry {
            system: guest_system,
            network: system.network.clone().unwrap_or_default(),
            streams: HashMap::new(),
            app_statuses: HashMap::new(),
            agent_connected: false,
            smart_home: None,
        };

        let guest_redacted = FullTelemetry {
            system: AgentTelemetry::default(),
            network: NetworkTelemetry::default(),
            streams: HashMap::new(),
            app_statuses: HashMap::new(),
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
