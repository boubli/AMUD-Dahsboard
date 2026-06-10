use crate::auth::agent_authenticated;
use crate::models::{
    ActionResult, ActionResultMsg, AgentTelemetry, AppState, PveTestResult, Webhook,
};
use crate::webhooks::{check_container_alerts, send_webhook_notification};
use serde::Deserialize;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path as FilePath;
use std::sync::Arc;
use tokio::net::TcpListener as TokioTcpListener;
#[cfg(unix)]
use tokio::net::UnixListener as TokioUnixListener;

pub(crate) fn handle_new_telemetry(state: &Arc<AppState>, metrics: AgentTelemetry) {
    let old_metrics = {
        let lock = state.latest_telemetry.read().unwrap();
        lock.clone()
    };
    if !old_metrics.lxc_containers.is_empty() {
        check_container_alerts(&old_metrics, &metrics, state);
    }
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
        let event_type = if connected { "agent_connected" } else { "agent_disconnected" };

        let webhooks = {
            let db = state.db.lock().unwrap();
            let mut stmt = db.prepare("SELECT id, name, url, event_types, is_active FROM webhooks WHERE is_active = 1").unwrap();
            let mut rows = stmt.query([]).unwrap();
            let mut list = Vec::new();
            while let Some(row) = rows.next().unwrap() {
                let id: i64 = row.get(0).unwrap();
                let name: String = row.get(1).unwrap();
                let url: String = row.get(2).unwrap();
                let event_types: String = row.get(3).unwrap();
                let is_active: i32 = row.get(4).unwrap();

                if event_types.split(',').any(|e| e.trim() == event_type) {
                    list.push(Webhook { id, name, url, event_types, is_active });
                }
            }
            list
        };

        let status_text = if connected { "online" } else { "offline" };

        for wh in webhooks {
            let url = wh.url.clone();
            let name = wh.name.clone();
            let event = event_type.to_string();
            let status_str = status_text.to_string();

            tokio::spawn(async move {
                send_webhook_notification(
                    url,
                    name,
                    &event,
                    "AMUD-Agent Daemon",
                    0,
                    &status_str,
                    "System",
                )
                .await;
            });
        }
    }
}

// Metrics collector listener task
pub(crate) fn start_agent_listener(state: Arc<AppState>) {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            let socket_path = std::env::var("AMUD_SOCKET_PATH")
                .unwrap_or_else(|_| "/opt/amud/run/amud.sock".to_string());
            run_uds_listener(
                &socket_path,
                state.clone(),
            )
            .await;
        }

        #[cfg(windows)]
        {
            let addr =
                std::env::var("AMUD_TCP_ADDR").unwrap_or_else(|_| "127.0.0.1:8050".to_string());
            run_tcp_listener(
                &addr,
                state.clone(),
            )
            .await;
        }
    });
}


#[cfg(unix)]
async fn run_uds_listener(
    path: &str,
    state: Arc<AppState>,
) {
    let uds_path = if FilePath::new(path)
        .parent()
        .map(|p| p.exists())
        .unwrap_or(false)
    {
        path
    } else {
        "/tmp/amud.sock"
    };

    println!(
        "Starting agent listener via UNIX Domain Socket at {}",
        uds_path
    );
    fs::remove_file(uds_path).ok();

    let listener = match TokioUnixListener::bind(uds_path) {
        Ok(l) => {
            fs::set_permissions(uds_path, std::fs::Permissions::from_mode(0o660)).ok();
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
            handle_agent_connection_change(&state, true);

            let (reader, mut writer) = stream.into_split();
            let state_clone = state.clone();

            // Set up communication channel
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            *state.agent_command_tx.lock().unwrap() = Some(tx.clone());

            // Spawn writer task
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                while let Some(cmd) = rx.recv().await {
                    if let Err(e) = writer.write_all(cmd.as_bytes()).await {
                        eprintln!("Failed to write command to UDS: {}", e);
                        break;
                    }
                    if let Err(e) = writer.flush().await {
                        eprintln!("Failed to flush command to UDS: {}", e);
                        break;
                    }
                }
            });

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut reader = tokio::io::BufReader::new(reader);
                let mut line = String::new();
                let mut authenticated = state_clone.agent_secret.is_empty();

                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break; // EOF
                    }

                    if !authenticated {
                        if agent_authenticated(&state_clone.agent_secret, &line) {
                            authenticated = true;
                            line.clear();
                            continue;
                        }
                        println!("AMUD-Agent rejected: invalid IPC authentication.");
                        break;
                    }

                    process_agent_line(&state_clone, &tx, &line);
                    line.clear();
                }
                println!("AMUD-Agent telemetry client disconnected.");
                handle_agent_connection_change(&state_clone, false);
                *state_clone.agent_command_tx.lock().unwrap() = None; // clear command tx
            });
        }
    }
}

pub(crate) fn process_agent_line(
    state: &Arc<AppState>,
    tx: &tokio::sync::mpsc::UnboundedSender<String>,
    line: &str,
) {
    #[derive(Deserialize)]
    struct ConfigReq {
        request: String,
    }
    #[derive(Deserialize)]
    struct PveTestMsg {
        test_pve_result: PveTestResult,
    }

    if let Ok(req) = serde_json::from_str::<ConfigReq>(line) {
        if req.request == "get_config" {
            let token = {
                let db_lock = state.db.lock().unwrap();
                let mut stmt = db_lock
                    .prepare("SELECT value FROM settings WHERE key = 'pve_api_token'")
                    .unwrap();
                stmt.query_row([], |row| row.get::<_, String>(0))
                    .unwrap_or_default()
            };
            let config_payload = serde_json::json!({
                "config": {
                    "pve_api_token": token
                }
            });
            if let Ok(mut serialized) = serde_json::to_vec(&config_payload) {
                serialized.push(b'\n');
                let _ = tx.send(String::from_utf8_lossy(&serialized).into_owned());
            }
        }
    } else if let Ok(msg) = serde_json::from_str::<ActionResultMsg>(line) {
        state
            .action_results
            .write()
            .unwrap()
            .insert(
                msg.action_result.request_id,
                ActionResult {
                    success: msg.action_result.success,
                    error: msg.action_result.error,
                },
            );
    } else if let Ok(msg) = serde_json::from_str::<PveTestMsg>(line) {
        *state.pve_test_response.write().unwrap() = Some(msg.test_pve_result);
    } else if let Ok(metrics) = serde_json::from_str::<AgentTelemetry>(line) {
        handle_new_telemetry(state, metrics);
    }
}

// For fallback or cross-compiles
#[cfg(not(unix))]
#[allow(dead_code)]
async fn run_uds_listener(
    _path: &str,
    _state: Arc<AppState>,
) {
}

#[allow(dead_code)]
async fn run_tcp_listener(
    addr: &str,
    state: Arc<AppState>,
) {
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
            handle_agent_connection_change(&state, true);

            let (reader, mut writer) = stream.into_split();
            let state_clone = state.clone();

            // Set up communication channel
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            *state.agent_command_tx.lock().unwrap() = Some(tx.clone());

            // Spawn writer task
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                while let Some(cmd) = rx.recv().await {
                    if let Err(e) = writer.write_all(cmd.as_bytes()).await {
                        eprintln!("Failed to write command to TCP: {}", e);
                        break;
                    }
                    if let Err(e) = writer.flush().await {
                        eprintln!("Failed to flush command to TCP: {}", e);
                        break;
                    }
                }
            });

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut reader = tokio::io::BufReader::new(reader);
                let mut line = String::new();
                let mut authenticated = state_clone.agent_secret.is_empty();

                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break; // EOF
                    }

                    if !authenticated {
                        if agent_authenticated(&state_clone.agent_secret, &line) {
                            authenticated = true;
                            line.clear();
                            continue;
                        }
                        println!("AMUD-Agent rejected: invalid IPC authentication.");
                        break;
                    }

                    process_agent_line(&state_clone, &tx, &line);
                    line.clear();
                }
                println!("AMUD-Agent telemetry client disconnected.");
                handle_agent_connection_change(&state_clone, false);
                *state_clone.agent_command_tx.lock().unwrap() = None; // clear command tx
            });
        }
    }
}
