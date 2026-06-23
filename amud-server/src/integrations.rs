use crate::models::App;
use crate::security::{sanitize_feed_link, sanitize_rss_feed_url};
use chrono::{DateTime, Utc};
use feed_rs::model::{Entry, Feed};
use serde_json::{json, Value};

fn looks_like_base64(s: &str) -> bool {
    !s.is_empty()
        && s.len().is_multiple_of(4)
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

pub(crate) fn adguard_basic_credential(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.contains(':') && !looks_like_base64(trimmed) {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(trimmed.as_bytes())
    } else {
        trimmed.to_string()
    }
}

fn adguard_blocked_today(json: &Value) -> u64 {
    json.get("num_blocked_filtering")
        .or_else(|| json.get("blocked_filtering"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .unwrap_or(0)
}
use std::time::Duration;

const RSS_MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const RSS_MAX_ENTRIES: usize = 3;
const RSS_MAX_TITLE_LEN: usize = 200;

fn build_client(accept_invalid_certs: bool) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn truncate_title(title: &str) -> String {
    if title.chars().count() <= RSS_MAX_TITLE_LEN {
        return title.to_string();
    }
    title.chars().take(RSS_MAX_TITLE_LEN).collect()
}

fn entry_sort_key(entry: &Entry) -> Option<DateTime<Utc>> {
    entry.published.or(entry.updated)
}

fn collect_feed_entries(feed: &Feed) -> Vec<Entry> {
    feed.entries.clone()
}

fn entry_to_json(entry: &Entry) -> Value {
    let title = entry
        .title
        .as_ref()
        .map(|t| truncate_title(&t.content))
        .unwrap_or_else(|| "Untitled".to_string());
    let raw_link = entry
        .links
        .first()
        .map(|l| l.href.as_str())
        .unwrap_or_default();
    let link = sanitize_feed_link(raw_link);
    let date_str = entry_sort_key(entry)
        .map(|d| d.format("%b %d").to_string())
        .unwrap_or_default();
    json!({
        "title": title,
        "link": link,
        "date": date_str,
    })
}

pub(crate) fn build_rss_entries(feed: &Feed) -> Vec<Value> {
    let mut entries = collect_feed_entries(feed);
    entries.sort_by(|a, b| {
        let a_key = entry_sort_key(a);
        let b_key = entry_sort_key(b);
        match (a_key, b_key) {
            (Some(a_dt), Some(b_dt)) => b_dt.cmp(&a_dt),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    entries
        .iter()
        .take(RSS_MAX_ENTRIES)
        .map(entry_to_json)
        .collect()
}

pub async fn fetch_integration_data(app: &App, accept_invalid_certs: bool) -> Option<Value> {
    if app.integration_type.is_empty() || app.api_key.is_empty() {
        return None;
    }

    let client = build_client(accept_invalid_certs);
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
            let credential = adguard_basic_credential(&app.api_key);
            let url = format!("{}/control/stats", base_url);
            let resp = client
                .get(&url)
                .header("Authorization", format!("Basic {}", credential))
                .send()
                .await
                .ok()?;
            if resp.status().is_success() {
                let json: Value = resp.json().await.ok()?;
                let status_url = format!("{}/control/status", base_url);
                let status_resp = client
                    .get(&status_url)
                    .header("Authorization", format!("Basic {}", credential))
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
                    "ads_blocked_today": adguard_blocked_today(&json),
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
        "overseerr" | "jellyseerr" => {
            let url = format!("{}/api/v1/request/count", base_url);
            let resp = client
                .get(&url)
                .header("X-Api-Key", &app.api_key)
                .send()
                .await
                .ok()?;
            if resp.status().is_success() {
                let json: Value = resp.json().await.ok()?;
                let pending = json.get("pending").and_then(|v| v.as_u64()).unwrap_or(0);
                return Some(json!({
                    "type": app.integration_type.as_str(),
                    "pending_requests": pending
                }));
            }
        }
        "prowlarr" => {
            let url = format!("{}/api/v1/indexer", base_url);
            let resp = client
                .get(&url)
                .header("X-Api-Key", &app.api_key)
                .send()
                .await
                .ok()?;
            if resp.status().is_success() {
                let indexers: Value = resp.json().await.ok()?;
                let (enabled, total) = count_prowlarr_indexers(&indexers);
                let queue_size = fetch_prowlarr_queue_size(&client, base_url, &app.api_key).await;
                return Some(json!({
                    "type": "prowlarr",
                    "indexers_enabled": enabled,
                    "indexers_total": total,
                    "queue_size": queue_size
                }));
            }
        }
        "uptime_kuma" => {
            return fetch_uptime_kuma(&client, base_url, &app.api_key).await;
        }
        "cloudflare_tunnel" => {
            return fetch_cloudflare_tunnel(&client, &app.api_key).await;
        }
        "peanut" => {
            return fetch_peanut(&client, base_url, &app.api_key).await;
        }
        "qbittorrent" => {
            return fetch_qbittorrent(accept_invalid_certs, base_url, &app.api_key).await;
        }
        "bazarr" => {
            return fetch_bazarr(&client, base_url, &app.api_key).await;
        }
        "rss" => {
            let feed_url = sanitize_rss_feed_url(&app.api_key)?;
            let resp = client.get(&feed_url).send().await.ok()?;
            if resp.status().is_success() {
                let bytes = resp.bytes().await.ok()?;
                if bytes.len() > RSS_MAX_RESPONSE_BYTES {
                    return None;
                }
                let feed = feed_rs::parser::parse(&bytes[..]).ok()?;
                let entries = build_rss_entries(&feed);
                return Some(json!({
                    "type": "rss",
                    "entries": entries,
                }));
            }
        }
        _ => return None,
    }

    None
}

fn count_prowlarr_indexers(indexers: &Value) -> (u64, u64) {
    let Some(arr) = indexers.as_array() else {
        return (0, 0);
    };
    let total = arr.len() as u64;
    let enabled = arr
        .iter()
        .filter(|i| i.get("enable").and_then(|v| v.as_bool()).unwrap_or(false))
        .count() as u64;
    (enabled, total)
}

async fn fetch_prowlarr_queue_size(client: &reqwest::Client, base_url: &str, api_key: &str) -> u64 {
    let url = format!("{}/api/v1/queue", base_url);
    let Ok(resp) = client.get(&url).header("X-Api-Key", api_key).send().await else {
        return 0;
    };
    if !resp.status().is_success() {
        return 0;
    }
    let Ok(json) = resp.json::<Value>().await else {
        return 0;
    };
    json.get("records")
        .and_then(|r| r.as_array())
        .map(|a| a.len() as u64)
        .or_else(|| json.get("totalRecords").and_then(|v| v.as_u64()))
        .unwrap_or(0)
}

pub(crate) fn parse_cloudflare_tunnel_creds(raw: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = raw.split('|').map(str::trim).collect();
    if parts.len() == 3 && parts.iter().all(|p| !p.is_empty()) {
        Some((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ))
    } else {
        None
    }
}

pub(crate) fn parse_uptime_kuma_heartbeats(json: &Value) -> (u64, u64) {
    let mut up = 0u64;
    let mut down = 0u64;
    let Some(hb_list) = json.get("heartbeatList").and_then(|v| v.as_object()) else {
        return (up, down);
    };
    for beats in hb_list.values() {
        let Some(arr) = beats.as_array() else {
            continue;
        };
        let Some(latest) = arr.last() else {
            continue;
        };
        if latest.get("status").and_then(|s| s.as_u64()) == Some(1) {
            up += 1;
        } else {
            down += 1;
        }
    }
    (up, down)
}

pub(crate) fn parse_peanut_stats(json: &Value) -> (String, String) {
    let charge = json
        .pointer("/ups/battery.charge")
        .or_else(|| json.get("battery.charge"))
        .and_then(|v| {
            v.as_str()
                .map(String::from)
                .or_else(|| v.as_i64().map(|n| n.to_string()))
        })
        .unwrap_or_else(|| "—".to_string());
    let raw_status = json
        .pointer("/ups/ups.status")
        .or_else(|| json.get("ups.status"))
        .or_else(|| json.get("status"))
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN");
    let status_label = match raw_status.to_ascii_uppercase().as_str() {
        "OL" | "ONLINE" => "Online",
        "OB" | "ON BATTERY" => "On battery",
        "LB" => "Low battery",
        "HB" => "High battery",
        "RB" => "Battery charging",
        "CHRG" => "Charging",
        _ => raw_status,
    };
    (charge, status_label.to_string())
}

async fn fetch_uptime_kuma(
    client: &reqwest::Client,
    base_url: &str,
    slug_or_token: &str,
) -> Option<Value> {
    let slug = slug_or_token.trim();
    if slug.is_empty() {
        return None;
    }
    let status_url = format!("{}/api/status-page/{}", base_url, slug);
    if let Ok(resp) = client.get(&status_url).send().await {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<Value>().await {
                let (up, down) = parse_uptime_kuma_heartbeats(&json);
                if up > 0 || down > 0 {
                    return Some(json!({
                        "type": "uptime_kuma",
                        "monitors_up": up,
                        "monitors_down": down
                    }));
                }
            }
        }
    }
    let monitors_url = format!("{}/api/monitors", base_url);
    let resp = client
        .get(&monitors_url)
        .header("Authorization", format!("Bearer {}", slug))
        .send()
        .await
        .ok()?;
    if resp.status().is_success() {
        let json: Value = resp.json().await.ok()?;
        let total = json.as_array().map(|a| a.len() as u64).unwrap_or(0);
        return Some(json!({
            "type": "uptime_kuma",
            "monitors_up": total,
            "monitors_down": 0
        }));
    }
    None
}

async fn fetch_cloudflare_tunnel(client: &reqwest::Client, creds_raw: &str) -> Option<Value> {
    let (account_id, tunnel_id, token) = parse_cloudflare_tunnel_creds(creds_raw)?;
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel/{}",
        account_id, tunnel_id
    );
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    let result = json.get("result")?;
    let status = result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let connections = result
        .get("connections")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(0);
    let name = result
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Tunnel");
    Some(json!({
        "type": "cloudflare_tunnel",
        "tunnel_name": name,
        "tunnel_status": status,
        "connections": connections
    }))
}

pub(crate) fn parse_qbittorrent_creds(raw: &str) -> Option<(String, String)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some((user, pass)) = trimmed.split_once('|') {
        let user = user.trim();
        let pass = pass.trim();
        if !user.is_empty() && !pass.is_empty() {
            return Some((user.to_string(), pass.to_string()));
        }
    }
    if let Some(idx) = trimmed.find(':') {
        let user = trimmed[..idx].trim();
        let pass = trimmed[idx + 1..].trim();
        if !user.is_empty() && !pass.is_empty() {
            return Some((user.to_string(), pass.to_string()));
        }
    }
    None
}

pub(crate) fn format_qbit_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec >= 1_000_000 {
        format!("{:.1} MB/s", bytes_per_sec as f64 / 1_000_000.0)
    } else if bytes_per_sec >= 1_000 {
        format!("{:.0} KB/s", bytes_per_sec as f64 / 1_000.0)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

pub(crate) fn count_qbittorrent_states(torrents: &Value) -> (u64, u64) {
    let Some(arr) = torrents.as_array() else {
        return (0, 0);
    };
    let mut downloading = 0u64;
    let mut seeding = 0u64;
    for t in arr {
        let Some(state) = t.get("state").and_then(|s| s.as_str()) else {
            continue;
        };
        match state {
            "downloading" | "stalledDL" | "metaDL" | "forcedDL" | "queuedDL" => downloading += 1,
            "uploading" | "stalledUP" | "forcedUP" | "queuedUP" => seeding += 1,
            _ => {}
        }
    }
    (downloading, seeding)
}

pub(crate) fn bazarr_wanted_count(json: &Value) -> u64 {
    json.get("data")
        .and_then(|d| d.as_array())
        .or_else(|| json.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(0)
}

async fn fetch_qbittorrent(
    accept_invalid_certs: bool,
    base_url: &str,
    creds_raw: &str,
) -> Option<Value> {
    let (username, password) = parse_qbittorrent_creds(creds_raw)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .cookie_store(true)
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .ok()?;
    let login_url = format!("{}/api/v2/auth/login", base_url.trim_end_matches('/'));
    let login = client
        .post(&login_url)
        .form(&[
            ("username", username.as_str()),
            ("password", password.as_str()),
        ])
        .send()
        .await
        .ok()?;
    if !login.status().is_success() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let transfer_resp = client
        .get(format!("{}/api/v2/transfer/info", base))
        .send()
        .await
        .ok()?;
    if !transfer_resp.status().is_success() {
        return None;
    }
    let transfer: Value = transfer_resp.json().await.ok()?;
    let dl_speed = transfer
        .get("dl_info_speed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let torrents_resp = client
        .get(format!("{}/api/v2/torrents/info", base))
        .send()
        .await
        .ok()?;
    if !torrents_resp.status().is_success() {
        return None;
    }
    let torrents: Value = torrents_resp.json().await.ok()?;
    let (downloading, seeding) = count_qbittorrent_states(&torrents);
    Some(json!({
        "type": "qbittorrent",
        "download_speed": format_qbit_speed(dl_speed),
        "active_downloads": downloading,
        "seeding": seeding
    }))
}

async fn fetch_bazarr(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    if api_key.trim().is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let movies_url = format!("{}/api/movies/wanted", base);
    let episodes_url = format!("{}/api/episodes/wanted", base);
    let movies_resp = client
        .get(&movies_url)
        .header("X-Api-Key", api_key)
        .send()
        .await
        .ok()?;
    let episodes_resp = client
        .get(&episodes_url)
        .header("X-Api-Key", api_key)
        .send()
        .await
        .ok()?;
    if !movies_resp.status().is_success() || !episodes_resp.status().is_success() {
        return None;
    }
    let movies: Value = movies_resp.json().await.ok()?;
    let episodes: Value = episodes_resp.json().await.ok()?;
    Some(json!({
        "type": "bazarr",
        "missing_movies": bazarr_wanted_count(&movies),
        "missing_episodes": bazarr_wanted_count(&episodes)
    }))
}

async fn fetch_peanut(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    for path in ["/api/v1/stats", "/api/stats"] {
        let url = format!("{}{}", base_url, path);
        let mut req = client.get(&url);
        if !api_key.is_empty() && api_key != "none" {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        let resp = req.send().await.ok()?;
        if resp.status().is_success() {
            let json: Value = resp.json().await.ok()?;
            let (battery, status) = parse_peanut_stats(&json);
            return Some(json!({
                "type": "peanut",
                "battery_percent": battery,
                "ups_status": status
            }));
        }
    }
    None
}

pub async fn execute_integration_action(
    app: &App,
    action: &str,
    accept_invalid_certs: bool,
) -> Option<Value> {
    if app.integration_type.is_empty() || app.api_key.is_empty() {
        return None;
    }

    let client = build_client(accept_invalid_certs);
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
            let credential = adguard_basic_credential(&app.api_key);
            let url = format!("{}/control/protection", base_url);
            let resp = client
                .post(&url)
                .header("Authorization", format!("Basic {}", credential))
                .json(&json!({"protection_enabled": false, "duration": 300000}))
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test</title>
    <item>
      <title>Older Story</title>
      <link>https://example.com/older</link>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Newer Story</title>
      <link>javascript:alert(1)</link>
      <pubDate>Wed, 15 Jan 2025 12:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Mid Story</title>
      <link>https://example.com/mid</link>
      <pubDate>Tue, 10 Jun 2025 12:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Fourth Story</title>
      <link>https://example.com/fourth</link>
      <pubDate>Thu, 20 Jun 2025 12:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>"#;

    #[test]
    fn build_rss_entries_sorts_and_limits() {
        let feed = feed_rs::parser::parse(SAMPLE_RSS.as_bytes()).expect("parse rss");
        let entries = build_rss_entries(&feed);
        assert_eq!(entries.len(), RSS_MAX_ENTRIES);
        assert_eq!(entries[0]["title"], "Fourth Story");
        assert_eq!(entries[1]["title"], "Mid Story");
        assert_eq!(entries[2]["title"], "Newer Story");
    }

    #[test]
    fn build_rss_entries_strips_malicious_links() {
        let feed = feed_rs::parser::parse(SAMPLE_RSS.as_bytes()).expect("parse rss");
        let entries = build_rss_entries(&feed);
        let newer = entries
            .iter()
            .find(|e| e["title"] == "Newer Story")
            .expect("newer story");
        assert_eq!(newer["link"], "");
    }

    #[test]
    fn build_rss_entries_parses_minimal_feed() {
        let feed = feed_rs::parser::parse(SAMPLE_RSS.as_bytes()).expect("parse rss");
        let entries = build_rss_entries(&feed);
        assert!(!entries.is_empty());
        assert_eq!(entries[0]["link"], "https://example.com/fourth");
    }

    #[test]
    fn parse_cloudflare_tunnel_creds_splits_pipe_format() {
        let parsed = parse_cloudflare_tunnel_creds("acc123|tunnel456|token789").expect("creds");
        assert_eq!(parsed.0, "acc123");
        assert_eq!(parsed.1, "tunnel456");
        assert_eq!(parsed.2, "token789");
        assert!(parse_cloudflare_tunnel_creds("bad").is_none());
    }

    #[test]
    fn parse_uptime_kuma_heartbeats_counts_up_down() {
        let json: Value = serde_json::json!({
            "heartbeatList": {
                "1": [{ "status": 1 }, { "status": 1 }],
                "2": [{ "status": 1 }],
                "3": [{ "status": 0 }]
            }
        });
        assert_eq!(parse_uptime_kuma_heartbeats(&json), (2, 1));
    }

    #[test]
    fn parse_peanut_stats_reads_battery_and_status() {
        let json: Value = serde_json::json!({
            "ups": {
                "battery.charge": "87",
                "ups.status": "OL"
            }
        });
        let (battery, status) = parse_peanut_stats(&json);
        assert_eq!(battery, "87");
        assert_eq!(status, "Online");
    }

    #[test]
    fn count_prowlarr_indexers_enabled_total() {
        let json: Value = serde_json::json!([
            { "enable": true },
            { "enable": false },
            { "enable": true }
        ]);
        assert_eq!(count_prowlarr_indexers(&json), (2, 3));
    }

    #[test]
    fn adguard_basic_credential_encodes_raw_user_pass() {
        let encoded = adguard_basic_credential("admin:secret");
        assert_ne!(encoded, "admin:secret");
        assert!(looks_like_base64(&encoded));
    }

    #[test]
    fn adguard_blocked_today_reads_num_blocked_filtering() {
        let json: Value = serde_json::json!({ "num_blocked_filtering": 42 });
        assert_eq!(adguard_blocked_today(&json), 42);
    }

    #[test]
    fn parse_qbittorrent_creds_pipe_and_colon() {
        assert_eq!(
            parse_qbittorrent_creds("admin|secret"),
            Some(("admin".into(), "secret".into()))
        );
        assert_eq!(
            parse_qbittorrent_creds("admin:secret"),
            Some(("admin".into(), "secret".into()))
        );
    }

    #[test]
    fn format_qbit_speed_human_readable() {
        assert_eq!(format_qbit_speed(500), "500 B/s");
        assert_eq!(format_qbit_speed(1500), "2 KB/s");
    }

    #[test]
    fn count_qbittorrent_states_groups_active() {
        let json: Value = serde_json::json!([
            { "state": "downloading" },
            { "state": "uploading" },
            { "state": "pausedUP" }
        ]);
        assert_eq!(count_qbittorrent_states(&json), (1, 1));
    }

    #[test]
    fn bazarr_wanted_count_reads_data_array() {
        let json: Value = serde_json::json!({ "data": [{}, {}] });
        assert_eq!(bazarr_wanted_count(&json), 2);
    }
}
