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

pub(crate) fn format_bytes_short(bytes: u64) -> String {
    if bytes >= 1_000_000_000_000 {
        format!("{:.1} TB", bytes as f64 / 1_000_000_000_000.0)
    } else if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.0} MB", bytes as f64 / 1_000_000.0)
    } else if bytes > 0 {
        format!("{:.0} KB", bytes as f64 / 1_000.0)
    } else {
        "—".to_string()
    }
}

pub(crate) fn arr_missing_count(json: &Value) -> u64 {
    json.get("totalRecords")
        .and_then(|v| v.as_u64())
        .or_else(|| {
            json.get("records")
                .and_then(|r| r.as_array())
                .map(|a| a.len() as u64)
        })
        .unwrap_or(0)
}

pub(crate) fn arr_array_len(json: &Value) -> u64 {
    json.as_array().map(|a| a.len() as u64).unwrap_or(0)
}

pub(crate) fn arr_disk_free(json: &Value) -> String {
    let Some(arr) = json.as_array() else {
        return "—".to_string();
    };
    let free: u64 = arr
        .iter()
        .filter_map(|d| d.get("freeSpace").and_then(|v| v.as_u64()))
        .sum();
    if free == 0 {
        "—".to_string()
    } else {
        format_bytes_short(free)
    }
}

pub(crate) fn arr_version(json: &Value) -> String {
    json.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("—")
        .to_string()
}

pub(crate) fn sonarr_episode_count(series_json: &Value) -> u64 {
    let Some(arr) = series_json.as_array() else {
        return 0;
    };
    arr.iter()
        .filter_map(|s| {
            s.pointer("/statistics/episodeCount")
                .and_then(|v| v.as_u64())
        })
        .sum()
}

pub(crate) fn pihole_summary_fields(json: &Value) -> (u64, u64, String, u64) {
    let blocked = json
        .get("ads_blocked_today")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .unwrap_or(0);
    let queries = json
        .get("dns_queries_today")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .unwrap_or(0);
    let pct = json
        .get("ads_percentage_today")
        .map(|v| {
            if let Some(f) = v.as_f64() {
                format!("{:.1}%", f)
            } else if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                "—".to_string()
            }
        })
        .unwrap_or_else(|| "—".to_string());
    let domains = json
        .get("domains_being_blocked")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .unwrap_or(0);
    (blocked, queries, pct, domains)
}

pub(crate) fn adguard_queries_today(json: &Value) -> u64 {
    json.get("num_dns_queries")
        .or_else(|| json.get("dns_queries"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .unwrap_or(0)
}

pub(crate) fn adguard_avg_time(json: &Value) -> String {
    json.get("avg_processing_time")
        .map(|v| {
            if let Some(f) = v.as_f64() {
                format!("{:.0} ms", f * 1000.0)
            } else if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                "—".to_string()
            }
        })
        .unwrap_or_else(|| "—".to_string())
}

async fn arr_api_get(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    path: &str,
) -> Option<Value> {
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let resp = client
        .get(&url)
        .header("X-Api-Key", api_key)
        .send()
        .await
        .ok()?;
    if resp.status().is_success() {
        resp.json().await.ok()
    } else {
        None
    }
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
            let url = format!("{}/admin/api.php?summaryRaw&auth={}", base_url, app.api_key);
            let resp = client.get(&url).send().await.ok()?;
            if resp.status().is_success() {
                let json: Value = resp.json().await.ok()?;
                let (blocked, queries, pct, domains) = pihole_summary_fields(&json);
                return Some(json!({
                    "type": "pihole",
                    "ads_blocked_today": blocked,
                    "dns_queries_today": queries,
                    "ads_percentage_today": pct,
                    "domains_being_blocked": domains,
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
                    "dns_queries_today": adguard_queries_today(&json),
                    "avg_processing_time": adguard_avg_time(&json),
                    "status": if is_running { "enabled" } else { "disabled" }
                }));
            }
        }
        "radarr" => {
            let key = app.api_key.clone();
            let (queue, missing, movies, disk, status) = tokio::join!(
                arr_api_get(&client, base_url, &key, "/api/v3/queue"),
                arr_api_get(&client, base_url, &key, "/api/v3/wanted/missing"),
                arr_api_get(&client, base_url, &key, "/api/v3/movie"),
                arr_api_get(&client, base_url, &key, "/api/v3/diskspace"),
                arr_api_get(&client, base_url, &key, "/api/v3/system/status"),
            );
            let queue_size = queue
                .as_ref()
                .and_then(|j| j.get("records"))
                .and_then(|r| r.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            return Some(json!({
                "type": "radarr",
                "queue_size": queue_size,
                "missing": missing.as_ref().map(arr_missing_count).unwrap_or(0),
                "library_count": movies.as_ref().map(arr_array_len).unwrap_or(0),
                "disk_free": disk.as_ref().map(arr_disk_free).unwrap_or_else(|| "—".to_string()),
                "version": status.as_ref().map(arr_version).unwrap_or_else(|| "—".to_string()),
            }));
        }
        "sonarr" => {
            let key = app.api_key.clone();
            let (queue, missing, series, disk, status) = tokio::join!(
                arr_api_get(&client, base_url, &key, "/api/v3/queue"),
                arr_api_get(&client, base_url, &key, "/api/v3/wanted/missing"),
                arr_api_get(&client, base_url, &key, "/api/v3/series"),
                arr_api_get(&client, base_url, &key, "/api/v3/diskspace"),
                arr_api_get(&client, base_url, &key, "/api/v3/system/status"),
            );
            let queue_size = queue
                .as_ref()
                .and_then(|j| j.get("records"))
                .and_then(|r| r.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            return Some(json!({
                "type": "sonarr",
                "queue_size": queue_size,
                "missing": missing.as_ref().map(arr_missing_count).unwrap_or(0),
                "series_count": series.as_ref().map(arr_array_len).unwrap_or(0),
                "episode_count": series.as_ref().map(sonarr_episode_count).unwrap_or(0),
                "disk_free": disk.as_ref().map(arr_disk_free).unwrap_or_else(|| "—".to_string()),
                "version": status.as_ref().map(arr_version).unwrap_or_else(|| "—".to_string()),
            }));
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
                return Some(json!({
                    "type": app.integration_type.as_str(),
                    "pending_requests": json.get("pending").and_then(|v| v.as_u64()).unwrap_or(0),
                    "approved_requests": json.get("approved").and_then(|v| v.as_u64()).unwrap_or(0),
                    "processing_requests": json.get("processing").and_then(|v| v.as_u64()).unwrap_or(0),
                    "total_requests": json.get("total").and_then(|v| v.as_u64()).unwrap_or(0),
                }));
            }
        }
        "prowlarr" => {
            let key = app.api_key.clone();
            let (indexers, status) = tokio::join!(
                arr_api_get(&client, base_url, &key, "/api/v1/indexer"),
                arr_api_get(&client, base_url, &key, "/api/v1/system/status"),
            );
            let indexers = indexers?;
            let (enabled, total) = count_prowlarr_indexers(&indexers);
            let failed = count_prowlarr_failed_indexers(&indexers);
            let queue_size = fetch_prowlarr_queue_size(&client, base_url, &key).await;
            return Some(json!({
                "type": "prowlarr",
                "indexers_enabled": enabled,
                "indexers_total": total,
                "failed_indexers": failed,
                "queue_size": queue_size,
                "version": status.as_ref().map(arr_version).unwrap_or_else(|| "—".to_string()),
            }));
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

pub(crate) fn count_prowlarr_failed_indexers(indexers: &Value) -> u64 {
    let Some(arr) = indexers.as_array() else {
        return 0;
    };
    arr.iter()
        .filter(|i| {
            i.get("enable").and_then(|v| v.as_bool()).unwrap_or(false)
                && i.get("status")
                    .and_then(|v| v.as_u64())
                    .map(|s| s >= 2)
                    .unwrap_or(false)
        })
        .count() as u64
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

fn format_runtime_secs(secs: i64) -> String {
    if secs >= 3600 {
        format!("{}h", secs / 3600)
    } else if secs >= 60 {
        format!("{}m", secs / 60)
    } else {
        format!("{}s", secs)
    }
}

pub(crate) fn parse_peanut_stats(json: &Value) -> (String, String, String, String) {
    let charge = json
        .pointer("/ups/battery.charge")
        .or_else(|| json.get("battery.charge"))
        .and_then(|v| {
            v.as_str()
                .map(String::from)
                .or_else(|| v.as_i64().map(|n| n.to_string()))
        })
        .unwrap_or_else(|| "—".to_string());
    let load = json
        .pointer("/ups/ups.load")
        .or_else(|| json.get("ups.load"))
        .and_then(|v| {
            v.as_str()
                .map(String::from)
                .or_else(|| v.as_i64().map(|n| format!("{}%", n)))
        })
        .unwrap_or_else(|| "—".to_string());
    let runtime = json
        .pointer("/ups/battery.runtime")
        .or_else(|| json.get("battery.runtime"))
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                if let Ok(secs) = s.parse::<i64>() {
                    return Some(format_runtime_secs(secs));
                }
                return Some(s.to_string());
            }
            v.as_i64().map(format_runtime_secs)
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
    (charge, load, runtime, status_label.to_string())
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
                    let total = up + down;
                    return Some(json!({
                        "type": "uptime_kuma",
                        "monitors_up": up,
                        "monitors_down": down,
                        "monitors_total": total,
                        "maintenance": json.get("maintenanceList").and_then(|v| v.as_array()).map(|a| a.len() as u64).unwrap_or(0),
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
            "monitors_down": 0,
            "monitors_total": total,
            "maintenance": 0,
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
    let colo_count = result
        .get("connections")
        .and_then(|v| v.as_array())
        .map(|arr| {
            let mut colos = std::collections::HashSet::new();
            for c in arr {
                if let Some(colo) = c.get("colo_name").and_then(|v| v.as_str()) {
                    colos.insert(colo);
                }
            }
            colos.len() as u64
        })
        .unwrap_or(0);
    let name = result
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Tunnel");
    Some(json!({
        "type": "cloudflare_tunnel",
        "tunnel_name": name,
        "tunnel_status": status,
        "connections": connections,
        "colo_count": colo_count,
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
    let ul_speed = transfer
        .get("up_info_speed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let free_disk = transfer
        .get("free_space_on_disk")
        .and_then(|v| v.as_u64())
        .map(format_bytes_short)
        .unwrap_or_else(|| "—".to_string());
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
    let total_torrents = torrents.as_array().map(|a| a.len() as u64).unwrap_or(0);
    Some(json!({
        "type": "qbittorrent",
        "download_speed": format_qbit_speed(dl_speed),
        "upload_speed": format_qbit_speed(ul_speed),
        "active_downloads": downloading,
        "seeding": seeding,
        "free_disk": free_disk,
        "total_torrents": total_torrents,
    }))
}

async fn fetch_bazarr(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    if api_key.trim().is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let key = api_key.to_string();
    let (movies_resp, episodes_resp, status_resp) = tokio::join!(
        client
            .get(format!("{}/api/movies/wanted", base))
            .header("X-Api-Key", &key)
            .send(),
        client
            .get(format!("{}/api/episodes/wanted", base))
            .header("X-Api-Key", &key)
            .send(),
        client
            .get(format!("{}/api/system/status", base))
            .header("X-Api-Key", &key)
            .send(),
    );
    let (Ok(movies_resp), Ok(episodes_resp)) = (movies_resp, episodes_resp) else {
        return None;
    };
    if !movies_resp.status().is_success() || !episodes_resp.status().is_success() {
        return None;
    }
    let movies: Value = movies_resp.json().await.ok()?;
    let episodes: Value = episodes_resp.json().await.ok()?;
    let status_json = if let Ok(resp) = status_resp {
        if resp.status().is_success() {
            resp.json::<Value>().await.ok()
        } else {
            None
        }
    } else {
        None
    };
    let version = status_json
        .as_ref()
        .map(arr_version)
        .unwrap_or_else(|| "—".to_string());
    let health = status_json
        .as_ref()
        .and_then(|j| j.get("health").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_else(|| "—".to_string());
    Some(json!({
        "type": "bazarr",
        "missing_movies": bazarr_wanted_count(&movies),
        "missing_episodes": bazarr_wanted_count(&episodes),
        "version": version,
        "health": health,
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
            let (battery, load, runtime, status) = parse_peanut_stats(&json);
            return Some(json!({
                "type": "peanut",
                "battery_percent": battery,
                "ups_load": load,
                "battery_runtime": runtime,
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
                "ups.load": "42",
                "battery.runtime": "3600",
                "ups.status": "OL"
            }
        });
        let (battery, load, runtime, status) = parse_peanut_stats(&json);
        assert_eq!(battery, "87");
        assert_eq!(load, "42");
        assert_eq!(runtime, "1h");
        assert_eq!(status, "Online");
    }

    #[test]
    fn arr_missing_count_reads_total_records() {
        let json: Value = serde_json::json!({ "totalRecords": 5 });
        assert_eq!(arr_missing_count(&json), 5);
    }

    #[test]
    fn arr_disk_free_sums_free_space() {
        let json: Value = serde_json::json!([
            { "freeSpace": 1_000_000_000 },
            { "freeSpace": 2_000_000_000 }
        ]);
        assert_eq!(arr_disk_free(&json), "3.0 GB");
    }

    #[test]
    fn pihole_summary_fields_parse() {
        let json: Value = serde_json::json!({
            "ads_blocked_today": 100,
            "dns_queries_today": 5000,
            "ads_percentage_today": 2.0,
            "domains_being_blocked": 250000
        });
        let (blocked, queries, pct, domains) = pihole_summary_fields(&json);
        assert_eq!(blocked, 100);
        assert_eq!(queries, 5000);
        assert_eq!(pct, "2.0%");
        assert_eq!(domains, 250000);
    }

    #[test]
    fn count_prowlarr_failed_indexers_detects_failures() {
        let json: Value = serde_json::json!([
            { "enable": true, "status": 2 },
            { "enable": true, "status": 1 },
            { "enable": false, "status": 2 }
        ]);
        assert_eq!(super::count_prowlarr_failed_indexers(&json), 1);
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
