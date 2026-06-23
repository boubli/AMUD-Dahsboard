use crate::models::App;
use crate::security::{sanitize_feed_link, sanitize_rss_feed_url};
use chrono::{DateTime, Utc};
use feed_rs::model::{Entry, Feed};
use serde_json::{json, Value};
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
}
