use crate::auth::{agent_challenge_nonce, parse_agent_auth_proof, verify_agent_auth};
use crate::db::{load_active_webhooks_for_event, with_db};
use crate::models::{
    ActionResult, ActionResultMsg, AgentCommandHandle, AgentTelemetry, AppState, PveTestResult,
};
use crate::webhooks::{check_container_alerts, send_webhook_notification};
use serde::Deserialize;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::path::Path as FilePath;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt};
#[cfg(not(unix))]
use tokio::net::TcpListener as TokioTcpListener;
#[cfg(unix)]
use tokio::net::UnixListener as TokioUnixListener;

pub(crate) fn handle_new_telemetry(state: &Arc<AppState>, metrics: AgentTelemetry) {
    let old_metrics = {
        let lock = state.latest_telemetry.read().unwrap();
        lock.clone()
    };
    check_container_alerts(&old_metrics, &metrics, state);
    *state.latest_telemetry.write().unwrap() = metrics;
}

pub(crate) fn handle_agent_connection_change(state: &Arc<AppState>, connected: bool) {
    let was_connected = {
        let mut conn_lock = state.agent_connected.write().unwrap();
        let old = *conn_lock;
        *conn_lock = connected;
        old
    };

    if was_connected != connected {
        let event_type = if connected {
            "agent_connected"
        } else {
            "agent_disconnected"
        };

        let status_text = if connected { "online" } else { "offline" };
        let state = state.clone();
        let event = event_type.to_string();
        let status_str = status_text.to_string();
        tokio::spawn(async move {
            let accept_invalid = {
                let cache = state.settings_cache.read().unwrap();
                cache
                    .get("accept_invalid_certs")
                    .map(|s| s == "1")
                    .unwrap_or(false)
            };
            let allow_private = {
                let cache = state.settings_cache.read().unwrap();
                cache
                    .get("webhooks_allow_private_ips")
                    .map(|s| s == "1")
                    .unwrap_or(false)
            };
            let event_filter = event.clone();
            let webhooks = with_db(state.db.clone(), move |db| {
                load_active_webhooks_for_event(db, &event_filter)
            })
            .await;
            for wh in webhooks {
                let url = wh.url;
                let name = wh.name;
                let event = event.clone();
                let status_str = status_str.clone();
                tokio::spawn(async move {
                    send_webhook_notification(
                        url,
                        name,
                        &event,
                        "AMUD-Agent Daemon",
                        0,
                        &status_str,
                        "System",
                        accept_invalid,
                        allow_private,
                    )
                    .await;
                });
            }
        });
    }
}

pub(crate) fn start_agent_listener(state: Arc<AppState>) {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            let socket_path = std::env::var("AMUD_SOCKET_PATH")
                .unwrap_or_else(|_| "/opt/amud/run/amud.sock".to_string());
            run_uds_listener(&socket_path, state.clone()).await;
        }

        #[cfg(windows)]
        {
            let addr =
                std::env::var("AMUD_TCP_ADDR").unwrap_or_else(|_| "127.0.0.1:8050".to_string());
            run_tcp_listener(&addr, state.clone()).await;
        }
    });
}

#[cfg(unix)]
fn resolve_uds_path(path: &str) -> Option<String> {
    let parent_ok = FilePath::new(path)
        .parent()
        .map(|p| p.exists())
        .unwrap_or(false);
    if parent_ok {
        Some(path.to_string())
    } else {
        eprintln!(
            "AMUD socket directory missing for {} — create the bind mount path (SEC-029).",
            path
        );
        None
    }
}

async fn handle_agent_stream<R, W>(reader: R, mut writer: W, state: Arc<AppState>, label: &str)
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let conn_id = state.next_agent_conn_id.fetch_add(1, Ordering::SeqCst);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let nonce = agent_challenge_nonce();
    let challenge_line = format!("{{\"challenge\":\"{nonce}\"}}\n");

    tokio::spawn(async move {
        if writer.write_all(challenge_line.as_bytes()).await.is_err() {
            return;
        }
        if writer.flush().await.is_err() {
            return;
        }
        while let Some(cmd) = rx.recv().await {
            if writer.write_all(cmd.as_bytes()).await.is_err() {
                break;
            }
            if writer.flush().await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let label = label.to_string();
    let agent_secret = state.agent_secret.clone();
    tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::new(reader);
        let mut line = String::new();
        let mut authenticated = false;

        while let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 {
                break;
            }

            if !authenticated {
                if let Some(proof) = parse_agent_auth_proof(&line) {
                    if verify_agent_auth(&agent_secret, &nonce, &proof) {
                        authenticated = true;
                        *state_clone.agent_command_tx.lock().unwrap() = Some(AgentCommandHandle {
                            id: conn_id,
                            tx: tx.clone(),
                        });
                        handle_agent_connection_change(&state_clone, true);
                        line.clear();
                        continue;
                    }
                }
                println!("AMUD-Agent rejected: invalid IPC authentication ({label}).");
                break;
            }

            process_agent_line(&state_clone, &tx, &line);
            line.clear();
        }
        println!("AMUD-Agent telemetry client disconnected ({label}).");
        if authenticated {
            let mut guard = state_clone.agent_command_tx.lock().unwrap();
            if guard.as_ref().map(|h| h.id) == Some(conn_id) {
                *guard = None;
                handle_agent_connection_change(&state_clone, false);
            }
        }
    });
}

#[cfg(unix)]
async fn run_uds_listener(path: &str, state: Arc<AppState>) {
    let Some(uds_path) = resolve_uds_path(path) else {
        return;
    };

    println!(
        "Starting agent listener via UNIX Domain Socket at {}",
        uds_path
    );
    std::fs::remove_file(&uds_path).ok();

    let listener = match TokioUnixListener::bind(&uds_path) {
        Ok(l) => {
            std::fs::set_permissions(&uds_path, std::fs::Permissions::from_mode(0o660)).ok();
            l
        }
        Err(e) => {
            eprintln!("UDS bind failed: {}. Telemetry offline listener active.", e);
            return;
        }
    };

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            println!("AMUD-Agent telemetry client UDS stream accepted.");
            let (reader, writer) = stream.into_split();
            handle_agent_stream(reader, writer, state.clone(), "UDS").await;
        }
    }
}

pub(crate) fn agent_config_payload(
    settings: &std::collections::HashMap<String, String>,
    pve_token_override: Option<&str>,
) -> serde_json::Value {
    let token = pve_token_override
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .or_else(|| settings.get("pve_api_token").map(String::as_str))
        .unwrap_or("");
    let configured = !token.is_empty();
    serde_json::json!({
        "config": {
            "pve_api_token_configured": configured,
            "pve_api_token": token,
            "telemetry_external_ifaces": settings.get("telemetry_external_ifaces").cloned().unwrap_or_default(),
            "telemetry_internal_ifaces": settings.get("telemetry_internal_ifaces").cloned().unwrap_or_default(),
            "telemetry_disk_mounts": settings.get("telemetry_disk_mounts").cloned().unwrap_or_default(),
        }
    })
}

#[allow(dead_code)]
pub(crate) fn pve_config_payload(token: &str) -> serde_json::Value {
    let mut settings = std::collections::HashMap::new();
    settings.insert("pve_api_token".to_string(), token.to_string());
    agent_config_payload(&settings, Some(token))
}

pub(crate) fn push_agent_config(state: &Arc<AppState>, pve_token_override: Option<&str>) {
    let cache = state.settings_cache.read().unwrap();
    let payload = agent_config_payload(&cache, pve_token_override);
    if let Ok(mut serialized) = serde_json::to_vec(&payload) {
        serialized.push(b'\n');
        if let Some(tx) = &*state.agent_command_tx.lock().unwrap() {
            let _ = tx
                .tx
                .send(String::from_utf8_lossy(&serialized).into_owned());
        }
    }
}

pub(crate) fn process_agent_line(
    state: &Arc<AppState>,
    tx: &tokio::sync::mpsc::UnboundedSender<String>,
    line: &str,
) {
    #[derive(Deserialize)]
    struct PveTestMsg {
        test_pve_result: PveTestResult,
    }

    if let Ok(req) = serde_json::from_str::<amud_protocol::ConfigRequest>(line) {
        if req.request == "get_config" {
            let cache = state.settings_cache.read().unwrap();
            let env_configured = req.pve_token_configured.unwrap_or(false);
            let token_override = if env_configured { Some("") } else { None };
            let mut config_payload = agent_config_payload(&cache, token_override);
            if env_configured {
                if let Some(config) = config_payload.get_mut("config") {
                    config["pve_api_token_configured"] = serde_json::json!(true);
                }
            }
            if let Ok(mut serialized) = serde_json::to_vec(&config_payload) {
                serialized.push(b'\n');
                let _ = tx.send(String::from_utf8_lossy(&serialized).into_owned());
            }
        }
    } else if let Ok(msg) = serde_json::from_str::<ActionResultMsg>(line) {
        state.action_results.write().unwrap().insert(
            msg.action_result.request_id,
            ActionResult {
                success: msg.action_result.success,
                error: msg.action_result.error,
                at: Instant::now(),
            },
        );
    } else if let Ok(msg) = serde_json::from_str::<PveTestMsg>(line) {
        *state.pve_test_response.write().unwrap() = Some(msg.test_pve_result);
    } else if let Ok(metrics) = serde_json::from_str::<AgentTelemetry>(line) {
        handle_new_telemetry(state, metrics);
    }
}

#[cfg(not(unix))]
#[allow(dead_code)]
async fn run_uds_listener(_path: &str, _state: Arc<AppState>) {}

#[cfg(not(unix))]
async fn run_tcp_listener(addr: &str, state: Arc<AppState>) {
    println!("Starting agent listener via TCP loopback on {}", addr);
    let listener = match TokioTcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("TCP loopback bind failed: {}.", e);
            return;
        }
    };

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            println!("AMUD-Agent telemetry client TCP stream accepted.");
            let (reader, writer) = stream.into_split();
            handle_agent_stream(reader, writer, state.clone(), "TCP").await;
        }
    }
}
