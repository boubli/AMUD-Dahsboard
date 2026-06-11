use crate::apps::{is_jellyfin_app, is_plex_app};
use crate::db::load_apps_from_db;
use crate::models::{AppState, AppStatus, AgentTelemetry, LxcContainer, Webhook};
use crate::security::url_allowed_for_health_check;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

pub(crate) fn start_status_poller(
    db: Arc<Mutex<Connection>>,
    app_statuses: Arc<RwLock<HashMap<String, AppStatus>>>,
) {
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .redirect(reqwest::redirect::Policy::none())
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        loop {
            let apps = {
                let db = db.lock().unwrap();
                let mut apps = Vec::<(String, String)>::new();
                if let Ok(mut stmt) = db.prepare("SELECT name, url FROM apps") {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    }) {
                        for app in rows.flatten() {
                            apps.push(app);
                        }
                    }
                }
                apps
            };

            let mut next = HashMap::new();
            for (name, url) in apps {
                let started = Instant::now();
                let status = if !url_allowed_for_health_check(&url) {
                    AppStatus {
                        status: "BLOCKED".to_string(),
                        latency_ms: None,
                    }
                } else {
                    match client.get(&url).send().await {
                        Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                            AppStatus {
                                status: "ONLINE".to_string(),
                                latency_ms: Some(started.elapsed().as_millis()),
                            }
                        }
                        Ok(_) | Err(_) => AppStatus {
                            status: "OFFLINE".to_string(),
                            latency_ms: None,
                        },
                    }
                };
                next.insert(name.to_lowercase(), status);
            }

            *app_statuses.write().unwrap() = next;
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });
}

pub(crate) async fn send_webhook_notification(
    url: String,
    name: String,
    event_type: &str,
    container_name: &str,
    vmid: i64,
    status: &str,
    provider: &str,
) {
    let client = reqwest::Client::new();
    let is_discord = url.contains("discord.com/api/webhooks/");
    let is_telegram = url.contains("api.telegram.org/bot");

    let response = if is_discord {
        let title = if event_type == "test" {
            "ðŸ”” AMUD Webhook Test".to_string()
        } else if status == "running" {
            format!("ðŸŸ¢ Container Started: {}", container_name)
        } else {
            format!("ðŸ”´ Container Stopped: {}", container_name)
        };

        let desc = if event_type == "test" {
            "Your AMUD Webhook Alerts Engine is successfully configured and ready to notify!".to_string()
        } else {
            format!("Container **{}** is now **{}**.", container_name, status)
        };

        let color = if event_type == "test" {
            0x2ecc71
        } else if status == "running" {
            0x10b981
        } else {
            0xef4444
        };

        let mut fields = vec![];
        if event_type != "test" {
            fields.push(serde_json::json!({
                "name": "Provider",
                "value": provider,
                "inline": true
            }));
            fields.push(serde_json::json!({
                "name": "VMID / ID",
                "value": vmid.to_string(),
                "inline": true
            }));
        }

        let payload = serde_json::json!({
            "username": "AMUD Alerts",
            "embeds": [{
                "title": title,
                "description": desc,
                "color": color,
                "fields": fields,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }]
        });

        client.post(&url).json(&payload).send().await
    } else if is_telegram {
        let text = if event_type == "test" {
            "<b>ðŸ”” AMUD Alert Test</b>\nYour Webhook Alerts Engine is successfully configured and ready to notify!".to_string()
        } else {
            let status_emoji = if status == "running" { "ðŸŸ¢" } else { "ðŸ”´" };
            format!(
                "{} <b>AMUD Alert: Container Status Changed</b>\n\n<b>Container:</b> <code>{}</code>\n<b>Status:</b> <code>{}</code>\n<b>Provider:</b> <code>{}</code>\n<b>VMID/ID:</b> <code>{}</code>",
                status_emoji, container_name, status.to_uppercase(), provider, vmid
            )
        };

        let parsed_url = reqwest::Url::parse(&url).ok();
        let chat_id = parsed_url.and_then(|u| {
            u.query_pairs()
                .find(|(k, _)| k == "chat_id")
                .map(|(_, v)| v.into_owned())
        });

        let payload = if let Some(cid) = chat_id {
            serde_json::json!({
                "chat_id": cid,
                "text": text,
                "parse_mode": "HTML"
            })
        } else {
            serde_json::json!({
                "text": text,
                "parse_mode": "HTML"
            })
        };

        client.post(&url).json(&payload).send().await
    } else {
        let payload = serde_json::json!({
            "event": event_type,
            "container": {
                "name": container_name,
                "vmid": vmid,
                "status": status,
                "provider": provider
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        client.post(&url).json(&payload).send().await
    };

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("Webhook '{}' successfully sent notification for '{}'", name, container_name);
            } else {
                eprintln!(
                    "Webhook '{}' failed with status code: {}. Body: {:?}",
                    name,
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to send webhook '{}': {}", name, e);
        }
    }
}

pub(crate) fn check_container_alerts(
    old_telemetry: &AgentTelemetry,
    new_telemetry: &AgentTelemetry,
    state: &Arc<AppState>,
) {
    let old_containers = &old_telemetry.lxc_containers;
    let new_containers = &new_telemetry.lxc_containers;

    let old_map: HashMap<i64, &LxcContainer> = old_containers
        .iter()
        .map(|c| (c.vmid, c))
        .collect();

    for new_c in new_containers {
        if let Some(old_c) = old_map.get(&new_c.vmid) {
            let old_status = &old_c.status;
            let new_status = &new_c.status;

            if old_status != new_status {
                let is_running_now = new_status == "running";
                let event_type = if is_running_now { "container_started" } else { "container_stopped" };
                let cooldown_key = format!("{}:{}", if new_c.vmid < 0 { "docker" } else { "lxc" }, new_c.name);

                {
                    let mut cooldowns = state.alert_cooldowns.lock().unwrap();
                    if let Some(&last_alert) = cooldowns.get(&cooldown_key) {
                        if last_alert.elapsed() < Duration::from_secs(60) {
                            println!("Alert for {} is suppressed due to cooldown", cooldown_key);
                            continue;
                        }
                    }
                    cooldowns.insert(cooldown_key.clone(), std::time::Instant::now());
                }

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

                        let subscribed = event_types.split(',').any(|e| e.trim() == event_type);
                        if subscribed {
                            list.push(Webhook { id, name, url, event_types, is_active });
                        }
                    }
                    list
                };

                let provider = if new_c.vmid < 0 { "Docker" } else { "Proxmox LXC" };
                for wh in webhooks {
                    let url = wh.url.clone();
                    let name = wh.name.clone();
                    let event = event_type.to_string();
                    let container_name = new_c.name.clone();
                    let vmid = new_c.vmid;
                    let status_str = new_status.clone();
                    let provider_str = provider.to_string();

                    tokio::spawn(async move {
                        send_webhook_notification(
                            url,
                            name,
                            &event,
                            &container_name,
                            vmid,
                            &status_str,
                            &provider_str,
                        )
                        .await;
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::{is_jellyfin_app, is_plex_app};
    use crate::media::default_media_streams;
    use crate::models::App;

    #[tokio::test]
    async fn test_check_container_alerts_transition() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS webhooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                event_types TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO webhooks (name, url, event_types, is_active) VALUES (?, ?, ?, ?)",
            params!["Test WH", "https://discord.com/api/webhooks/test", "container_stopped", 1],
        )
        .unwrap();

        let state = Arc::new(AppState {
            db: Arc::new(Mutex::new(conn)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            latest_telemetry: Arc::new(RwLock::new(AgentTelemetry::default())),
            agent_connected: Arc::new(RwLock::new(false)),
            media_streams: Arc::new(RwLock::new(default_media_streams())),
            app_statuses: Arc::new(RwLock::new(HashMap::new())),
            agent_command_tx: Arc::new(Mutex::new(None)),
            pve_test_response: Arc::new(RwLock::new(None)),
            action_results: Arc::new(RwLock::new(HashMap::new())),
            settings_cache: Arc::new(RwLock::new(HashMap::new())),
            alert_cooldowns: Arc::new(Mutex::new(HashMap::new())),
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            api_rate_limits: Arc::new(Mutex::new(HashMap::new())),
            agent_secret: Arc::new("test-secret".to_string()),
            smart_home_telemetry: Arc::new(RwLock::new(Default::default())),
        });

        let old_telemetry = AgentTelemetry {
            cpu_usage: 0,
            ram_usage: 0,
            ram_used_gb: 0.0,
            ram_total_gb: 0.0,
            cpu_temp: 0.0,
            disk_usage: 0,
            disk_used_gb: 0.0,
            disk_total_gb: 0.0,
            network: None,
            lxc_containers: vec![LxcContainer {
                vmid: 100,
                status: "running".to_string(),
                name: "test-lxc".to_string(),
                ..Default::default()
            }],
        };

        let new_telemetry = AgentTelemetry {
            cpu_usage: 0,
            ram_usage: 0,
            ram_used_gb: 0.0,
            ram_total_gb: 0.0,
            cpu_temp: 0.0,
            disk_usage: 0,
            disk_used_gb: 0.0,
            disk_total_gb: 0.0,
            network: None,
            lxc_containers: vec![LxcContainer {
                vmid: 100,
                status: "stopped".to_string(),
                name: "test-lxc".to_string(),
                ..Default::default()
            }],
        };

        // Trigger alert check
        check_container_alerts(&old_telemetry, &new_telemetry, &state);

        // Verify that it added the cooldown key (lxc:test-lxc) to alert_cooldowns map
        let cooldowns = state.alert_cooldowns.lock().unwrap();
        assert!(cooldowns.contains_key("lxc:test-lxc"));
    }

    #[tokio::test]
    async fn test_check_container_alerts_no_change() {

        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS webhooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                event_types TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )
        .unwrap();

        let state = Arc::new(AppState {
            db: Arc::new(Mutex::new(conn)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            latest_telemetry: Arc::new(RwLock::new(AgentTelemetry::default())),
            agent_connected: Arc::new(RwLock::new(false)),
            media_streams: Arc::new(RwLock::new(default_media_streams())),
            app_statuses: Arc::new(RwLock::new(HashMap::new())),
            agent_command_tx: Arc::new(Mutex::new(None)),
            pve_test_response: Arc::new(RwLock::new(None)),
            action_results: Arc::new(RwLock::new(HashMap::new())),
            settings_cache: Arc::new(RwLock::new(HashMap::new())),
            alert_cooldowns: Arc::new(Mutex::new(HashMap::new())),
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            api_rate_limits: Arc::new(Mutex::new(HashMap::new())),
            agent_secret: Arc::new("test-secret".to_string()),
            smart_home_telemetry: Arc::new(RwLock::new(Default::default())),
        });

        let old_telemetry = AgentTelemetry {
            lxc_containers: vec![LxcContainer {
                vmid: 100,
                status: "running".to_string(),
                name: "test-lxc".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let new_telemetry = AgentTelemetry {
            lxc_containers: vec![LxcContainer {
                vmid: 100,
                status: "running".to_string(),
                name: "test-lxc".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        check_container_alerts(&old_telemetry, &new_telemetry, &state);

        let cooldowns = state.alert_cooldowns.lock().unwrap();
        assert!(!cooldowns.contains_key("lxc:test-lxc"));
    }

    fn sample_app(name: &str, url: &str, icon: &str) -> App {
        App {
            id: 1,
            name: name.to_string(),
            url: url.to_string(),
            icon: icon.to_string(),
            description: String::new(),
            category: String::new(),
            node_tag: String::new(),
            mac_address: String::new(),
        }
    }

    #[test]
    fn media_stream_cards_require_registered_apps() {
        assert!(!is_jellyfin_app(&sample_app("Sonarr", "http://sonarr.local", "sonarr")));
        assert!(!is_jellyfin_app(&sample_app("Assembly", "http://assembly.local", "custom")));
        assert!(is_jellyfin_app(&sample_app("Media Server", "http://nas:8096", "jellyfin")));
        assert!(is_jellyfin_app(&sample_app("Jellyfin", "http://nas:8096", "jellyfin")));
        assert!(!is_plex_app(&sample_app("Sonarr", "http://sonarr.local", "sonarr")));
        assert!(is_plex_app(&sample_app("Plex", "http://plex.local:32400", "plex")));
    }
}
