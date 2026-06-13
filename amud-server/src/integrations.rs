use crate::models::App;
use serde_json::{json, Value};
use std::time::Duration;

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub async fn fetch_integration_data(app: &App) -> Option<Value> {
    if app.integration_type.is_empty() || app.api_key.is_empty() {
        return None;
    }

    let client = build_client();
    let base_url = app.url.trim_end_matches('/');

    match app.integration_type.as_str() {
        "pihole" => {
            // GET /admin/api.php?summaryRaw&auth=API_KEY
            let url = format!("{}/admin/api.php?summaryRaw&auth={}", base_url, app.api_key);
            let resp = client.get(&url).send().await.ok()?;
            if resp.status().is_success() {
                let json: Value = resp.json().await.ok()?;
                return Some(json!({
                    "type": "pihole",
                    "ads_blocked_today": json.get("ads_blocked_today").unwrap_or(&json!(0)),
                    "status": json.get("status").unwrap_or(&json!("unknown"))
                }));
            }
        }
        "adguard" => {
            // AdGuard Home API uses Basic Auth
            let url = format!("{}/control/stats", base_url);
            let resp = client
                .get(&url)
                .header("Authorization", format!("Basic {}", app.api_key))
                .send()
                .await
                .ok()?;
            if resp.status().is_success() {
                let json: Value = resp.json().await.ok()?;
                let status_url = format!("{}/control/status", base_url);
                let status_resp = client
                    .get(&status_url)
                    .header("Authorization", format!("Basic {}", app.api_key))
                    .send()
                    .await
                    .ok();
                let is_running = if let Some(s_resp) = status_resp {
                    if let Ok(s_json) = s_resp.json::<Value>().await {
                        s_json
                            .get("protection_enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true)
                    } else {
                        true
                    }
                } else {
                    true
                };

                return Some(json!({
                    "type": "adguard",
                    "ads_blocked_today": json.get("blocked_filtering").unwrap_or(&json!(0)),
                    "status": if is_running { "enabled" } else { "disabled" }
                }));
            }
        }
        "radarr" => {
            // Radarr queue
            let url = format!("{}/api/v3/queue", base_url);
            let resp = client
                .get(&url)
                .header("X-Api-Key", &app.api_key)
                .send()
                .await
                .ok()?;
            if resp.status().is_success() {
                let json: Value = resp.json().await.ok()?;
                let records = json
                    .get("records")
                    .and_then(|r| r.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                return Some(json!({
                    "type": "radarr",
                    "queue_size": records
                }));
            }
        }
        "sonarr" => {
            let url = format!("{}/api/v3/queue", base_url);
            let resp = client
                .get(&url)
                .header("X-Api-Key", &app.api_key)
                .send()
                .await
                .ok()?;
            if resp.status().is_success() {
                let json: Value = resp.json().await.ok()?;
                let records = json
                    .get("records")
                    .and_then(|r| r.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                return Some(json!({
                    "type": "sonarr",
                    "queue_size": records
                }));
            }
        }
        _ => return None,
    }

    None
}

pub async fn execute_integration_action(app: &App, action: &str) -> Option<Value> {
    if app.integration_type.is_empty() || app.api_key.is_empty() {
        return None;
    }

    let client = build_client();
    let base_url = app.url.trim_end_matches('/');

    match app.integration_type.as_str() {
        "pihole" if action == "disable" => {
            // GET /admin/api.php?disable=300&auth=API_KEY
            let url = format!(
                "{}/admin/api.php?disable=300&auth={}",
                base_url, app.api_key
            );
            let resp = client.get(&url).send().await.ok()?;
            if resp.status().is_success() {
                return Some(json!({"success": true}));
            }
        }
        "adguard" if action == "disable" => {
            // POST /control/protection
            let url = format!("{}/control/protection", base_url);
            let resp = client
                .post(&url)
                .header("Authorization", format!("Basic {}", app.api_key))
                .json(&json!({"protection_enabled": false, "duration": 300000})) // 300s
                .send()
                .await
                .ok()?;
            if resp.status().is_success() {
                return Some(json!({"success": true}));
            }
        }
        _ => {}
    }

    None
}
