use crate::models::App;
use crate::security::{get_rss_url_allowed, sanitize_feed_link, sanitize_rss_feed_url};
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

pub(crate) fn arr_health_label(json: &Value) -> String {
    if json.get("isHealthy").and_then(|v| v.as_bool()) == Some(true) {
        return "Healthy".to_string();
    }
    if json.get("isHealthy").and_then(|v| v.as_bool()) == Some(false) {
        return "Unhealthy".to_string();
    }
    json.get("health")
        .and_then(|v| v.as_str())
        .unwrap_or("—")
        .to_string()
}

pub(crate) fn adguard_block_pct(json: &Value) -> String {
    let blocked = adguard_blocked_today(json) as f64;
    let queries = adguard_queries_today(json) as f64;
    if queries <= 0.0 {
        return "—".to_string();
    }
    format!("{:.1}%", (blocked / queries) * 100.0)
}

pub(crate) fn adguard_rewrites(json: &Value) -> u64 {
    json.get("num_replaced_safebrowsing")
        .or_else(|| json.get("num_replaced_parental"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .unwrap_or(0)
}

pub(crate) fn pihole_extra_fields(json: &Value) -> (u64, String) {
    let clients = json
        .get("unique_clients")
        .or_else(|| json.get("clients_ever_seen"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .unwrap_or(0);
    let gravity = json
        .get("gravity_last_updated")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .map(|ts| {
            let dt = chrono::DateTime::from_timestamp(ts as i64, 0);
            dt.map(|d| d.format("%b %d").to_string())
                .unwrap_or_else(|| ts.to_string())
        })
        .unwrap_or_else(|| "—".to_string());
    (clients, gravity)
}

pub(crate) fn count_qbittorrent_paused(torrents: &Value) -> u64 {
    let Some(arr) = torrents.as_array() else {
        return 0;
    };
    arr.iter()
        .filter(|t| {
            t.get("state").and_then(|s| s.as_str()) == Some("pausedDL")
                || t.get("state").and_then(|s| s.as_str()) == Some("pausedUP")
        })
        .count() as u64
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

struct ArrFetchConfig {
    api_prefix: &'static str,
    library_path: &'static str,
    extra_library_path: Option<&'static str>,
}

struct ArrFetchResult {
    queue_size: u64,
    missing: u64,
    library_count: u64,
    extra_count: u64,
    disk_free: String,
    version: String,
    health: String,
    library_json: Option<Value>,
}

fn arr_queue_size(queue: &Option<Value>) -> u64 {
    queue
        .as_ref()
        .and_then(|j| j.get("records"))
        .and_then(|r| r.as_array())
        .map(|a| a.len())
        .unwrap_or(0) as u64
}

async fn fetch_arr_stats(
    client: &reqwest::Client,
    base_url: &str,
    key: &str,
    config: ArrFetchConfig,
) -> ArrFetchResult {
    let prefix = config.api_prefix;
    let queue_path = format!("{prefix}/queue");
    let missing_path = format!("{prefix}/wanted/missing");
    let library_path = format!("{prefix}{}", config.library_path);
    let disk_path = format!("{prefix}/diskspace");
    let status_path = format!("{prefix}/system/status");
    let extra_path = config.extra_library_path.map(|p| format!("{prefix}{p}"));

    let (queue, missing, library, disk, status, extra) = if let Some(ref extra_p) = extra_path {
        tokio::join!(
            arr_api_get(client, base_url, key, &queue_path),
            arr_api_get(client, base_url, key, &missing_path),
            arr_api_get(client, base_url, key, &library_path),
            arr_api_get(client, base_url, key, &disk_path),
            arr_api_get(client, base_url, key, &status_path),
            arr_api_get(client, base_url, key, extra_p),
        )
    } else {
        let (queue, missing, library, disk, status) = tokio::join!(
            arr_api_get(client, base_url, key, &queue_path),
            arr_api_get(client, base_url, key, &missing_path),
            arr_api_get(client, base_url, key, &library_path),
            arr_api_get(client, base_url, key, &disk_path),
            arr_api_get(client, base_url, key, &status_path),
        );
        (queue, missing, library, disk, status, None)
    };

    let extra_count = extra.as_ref().map(arr_array_len).unwrap_or(0);

    ArrFetchResult {
        queue_size: arr_queue_size(&queue),
        missing: missing.as_ref().map(arr_missing_count).unwrap_or(0),
        library_count: library.as_ref().map(arr_array_len).unwrap_or(0),
        extra_count,
        disk_free: disk
            .as_ref()
            .map(arr_disk_free)
            .unwrap_or_else(|| "—".to_string()),
        version: status
            .as_ref()
            .map(arr_version)
            .unwrap_or_else(|| "—".to_string()),
        health: status
            .as_ref()
            .map(arr_health_label)
            .unwrap_or_else(|| "—".to_string()),
        library_json: library,
    }
}

async fn fetch_lidarr_arr(client: &reqwest::Client, base_url: &str, key: &str) -> Option<Value> {
    let stats = fetch_arr_stats(
        client,
        base_url,
        key,
        ArrFetchConfig {
            api_prefix: "/api/v1",
            library_path: "/artist",
            extra_library_path: Some("/album"),
        },
    )
    .await;
    Some(json!({
        "type": "lidarr",
        "queue_size": stats.queue_size,
        "missing": stats.missing,
        "library_count": stats.library_count,
        "album_count": stats.extra_count,
        "disk_free": stats.disk_free,
        "version": stats.version,
        "health": stats.health,
    }))
}

async fn fetch_readarr_arr(client: &reqwest::Client, base_url: &str, key: &str) -> Option<Value> {
    let stats = fetch_arr_stats(
        client,
        base_url,
        key,
        ArrFetchConfig {
            api_prefix: "/api/v1",
            library_path: "/book",
            extra_library_path: Some("/author"),
        },
    )
    .await;
    Some(json!({
        "type": "readarr",
        "queue_size": stats.queue_size,
        "missing": stats.missing,
        "library_count": stats.library_count,
        "author_count": stats.extra_count,
        "disk_free": stats.disk_free,
        "version": stats.version,
        "health": stats.health,
    }))
}

async fn fetch_whisparr_arr(client: &reqwest::Client, base_url: &str, key: &str) -> Option<Value> {
    let stats = fetch_arr_stats(
        client,
        base_url,
        key,
        ArrFetchConfig {
            api_prefix: "/api/v3",
            library_path: "/series",
            extra_library_path: None,
        },
    )
    .await;
    let episode_count = stats
        .library_json
        .as_ref()
        .map(sonarr_episode_count)
        .unwrap_or(0);
    Some(json!({
        "type": "whisparr",
        "queue_size": stats.queue_size,
        "missing": stats.missing,
        "series_count": stats.library_count,
        "episode_count": episode_count,
        "disk_free": stats.disk_free,
        "version": stats.version,
        "health": stats.health,
    }))
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

pub(crate) fn extract_feed_icon(feed: &Feed) -> Option<String> {
    feed.logo
        .as_ref()
        .map(|img| img.uri.trim())
        .filter(|u| !u.is_empty())
        .map(|u| u.to_string())
        .or_else(|| {
            feed.icon
                .as_ref()
                .map(|img| img.uri.trim())
                .filter(|u| !u.is_empty())
                .map(|u| u.to_string())
        })
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
                let (unique_clients, gravity_updated) = pihole_extra_fields(&json);
                return Some(json!({
                    "type": "pihole",
                    "ads_blocked_today": blocked,
                    "dns_queries_today": queries,
                    "ads_percentage_today": pct,
                    "domains_being_blocked": domains,
                    "status": json.get("status").unwrap_or(&json!("unknown")),
                    "unique_clients": unique_clients,
                    "gravity_updated": gravity_updated,
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
                    "status": if is_running { "enabled" } else { "disabled" },
                    "block_pct": adguard_block_pct(&json),
                    "dns_rewrites": adguard_rewrites(&json),
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
                "health": status.as_ref().map(arr_health_label).unwrap_or_else(|| "—".to_string()),
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
        "lidarr" => return fetch_lidarr_arr(&client, base_url, &app.api_key).await,
        "readarr" => return fetch_readarr_arr(&client, base_url, &app.api_key).await,
        "whisparr" => return fetch_whisparr_arr(&client, base_url, &app.api_key).await,
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
                    "declined_requests": json.get("declined").and_then(|v| v.as_u64()).unwrap_or(0),
                    "available_requests": json.get("available").and_then(|v| v.as_u64()).unwrap_or(0),
                }));
            }
        }
        "prowlarr" => {
            let key = app.api_key.clone();
            let (indexers, status, health, apps) = tokio::join!(
                arr_api_get(&client, base_url, &key, "/api/v1/indexer"),
                arr_api_get(&client, base_url, &key, "/api/v1/system/status"),
                arr_api_get(&client, base_url, &key, "/api/v1/health"),
                arr_api_get(&client, base_url, &key, "/api/v1/applications"),
            );
            let indexers = indexers?;
            let (enabled, total) = count_prowlarr_indexers(&indexers);
            let failed = count_prowlarr_failed_indexers(&indexers);
            let queue_size = fetch_prowlarr_queue_size(&client, base_url, &key).await;
            let health_label = health
                .as_ref()
                .map(arr_health_label)
                .unwrap_or_else(|| "—".to_string());
            let app_count = apps
                .as_ref()
                .and_then(|v| v.as_array())
                .map(|a| a.len() as u64)
                .unwrap_or(0);
            return Some(json!({
                "type": "prowlarr",
                "indexers_enabled": enabled,
                "indexers_total": total,
                "failed_indexers": failed,
                "queue_size": queue_size,
                "version": status.as_ref().map(arr_version).unwrap_or_else(|| "—".to_string()),
                "health": health_label,
                "app_count": app_count,
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
        "sabnzbd" => return fetch_sabnzbd(&client, base_url, &app.api_key).await,
        "nzbget" => return fetch_nzbget(&client, base_url, &app.api_key).await,
        "transmission" => {
            return fetch_transmission(accept_invalid_certs, base_url, &app.api_key).await;
        }
        "jackett" => return fetch_jackett(&client, base_url, &app.api_key).await,
        "tautulli" => return fetch_tautulli(&client, base_url, &app.api_key).await,
        "audiobookshelf" => return fetch_audiobookshelf(&client, base_url, &app.api_key).await,
        "immich" => return fetch_immich(&client, base_url, &app.api_key).await,
        "tdarr" => return fetch_tdarr(&client, base_url, &app.api_key).await,
        "maintainerr" => return fetch_maintainerr(&client, base_url, &app.api_key).await,
        "frigate" => return fetch_frigate(&client, base_url, &app.api_key).await,
        "fritz" => {
            let fritz_client = crate::fritz::build_fritz_client(accept_invalid_certs);
            return crate::fritz::fetch_fritz(&fritz_client, base_url, &app.api_key).await;
        }
        "portainer" => {
            return crate::homelab::fetch_portainer(&client, base_url, &app.api_key).await;
        }
        "opnsense" => {
            return crate::homelab::fetch_opnsense(&client, base_url, &app.api_key).await;
        }
        "pfsense" => {
            return crate::homelab::fetch_pfsense(&client, base_url, &app.api_key).await;
        }
        "truenas" => {
            return crate::homelab::fetch_truenas(&client, base_url, &app.api_key).await;
        }
        "unifi" => {
            let unifi_client = crate::homelab::build_homelab_client(accept_invalid_certs);
            return crate::homelab::fetch_unifi(&unifi_client, base_url, &app.api_key).await;
        }
        "grafana" => {
            return crate::homelab::fetch_grafana(&client, base_url, &app.api_key).await;
        }
        "netdata" => {
            return crate::homelab::fetch_netdata(&client, base_url, &app.api_key).await;
        }
        "glances" => {
            return crate::homelab::fetch_glances(&client, base_url, &app.api_key).await;
        }
        "beszel" => {
            return crate::homelab::fetch_beszel(&client, base_url, &app.api_key).await;
        }
        "paperless" => {
            return crate::homelab::fetch_paperless(&client, base_url, &app.api_key).await;
        }
        "mealie" => {
            return crate::homelab::fetch_mealie(&client, base_url, &app.api_key).await;
        }
        "nextcloud" => {
            return crate::homelab::fetch_nextcloud(&client, base_url, &app.api_key).await;
        }
        "vaultwarden" => {
            return crate::homelab::fetch_vaultwarden(&client, base_url, &app.api_key).await;
        }
        "deluge" => {
            return crate::homelab::fetch_deluge(&client, base_url, &app.api_key).await;
        }
        "navidrome" => {
            return crate::homelab::fetch_navidrome(&client, base_url, &app.api_key).await;
        }
        "komga" => {
            return crate::homelab::fetch_komga(&client, base_url, &app.api_key).await;
        }
        "photoprism" => {
            return crate::homelab::fetch_photoprism(&client, base_url, &app.api_key).await;
        }
        "proxmox" => {
            return crate::homelab::fetch_proxmox(&client, base_url, &app.api_key).await;
        }
        "tailscale" => {
            return crate::homelab::fetch_tailscale(&client, base_url, &app.api_key).await;
        }
        "netbird" => {
            return crate::homelab::fetch_netbird(&client, base_url, &app.api_key).await;
        }
        "synology" => return crate::homelab::fetch_synology(&client, base_url, &app.api_key).await,
        "unraid" => return crate::homelab::fetch_unraid(&client, base_url, &app.api_key).await,
        "dockge" => return crate::homelab::fetch_dockge(&client, base_url, &app.api_key).await,
        "nginx_proxy_manager" => {
            return crate::homelab::fetch_nginx_proxy_manager(&client, base_url, &app.api_key)
                .await;
        }
        "traefik" => return crate::homelab::fetch_traefik(&client, base_url, &app.api_key).await,
        "authentik" => {
            return crate::homelab::fetch_authentik(&client, base_url, &app.api_key).await
        }
        "authelia" => return crate::homelab::fetch_authelia(&client, base_url, &app.api_key).await,
        "crowdsec" => return crate::homelab::fetch_crowdsec(&client, base_url, &app.api_key).await,
        "node_red" => return crate::homelab::fetch_node_red(&client, base_url, &app.api_key).await,
        "zigbee2mqtt" => {
            return crate::homelab::fetch_zigbee2mqtt(&client, base_url, &app.api_key).await
        }
        "homeassistant" => {
            return crate::homelab::fetch_homeassistant(&client, base_url, &app.api_key).await;
        }
        "emby" => return crate::homelab::fetch_emby(&client, base_url, &app.api_key).await,
        "scrypted" => return crate::homelab::fetch_scrypted(&client, base_url, &app.api_key).await,
        "mylar" => return crate::homelab::fetch_mylar(&client, base_url, &app.api_key).await,
        "kapowarr" => return crate::homelab::fetch_kapowarr(&client, base_url, &app.api_key).await,
        "huntarr" => return crate::homelab::fetch_huntarr(&client, base_url, &app.api_key).await,
        "proxmox_backup" => {
            return crate::homelab::fetch_proxmox_backup(&client, base_url, &app.api_key).await;
        }
        "technitium" => {
            return crate::homelab::fetch_technitium(&client, base_url, &app.api_key).await
        }
        "blocky" => return crate::homelab::fetch_blocky(&client, base_url, &app.api_key).await,
        "openwrt" => return crate::homelab::fetch_openwrt(&client, base_url, &app.api_key).await,
        t if crate::homelab::is_health_only(t) => {
            return crate::homelab::fetch_health_integration(&client, t, base_url, &app.api_key)
                .await;
        }
        "rss" => {
            let feed_url = sanitize_rss_feed_url(&app.api_key)?;
            let resp = get_rss_url_allowed(&client, &feed_url).await?;
            if resp.status().is_success() {
                let bytes = resp.bytes().await.ok()?;
                if bytes.len() > RSS_MAX_RESPONSE_BYTES {
                    return None;
                }
                let feed = feed_rs::parser::parse(&bytes[..]).ok()?;
                let entries = build_rss_entries(&feed);
                let feed_icon = extract_feed_icon(&feed);
                return Some(json!({
                    "type": "rss",
                    "entries": entries,
                    "feed_icon": feed_icon,
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

pub(crate) fn parse_uptime_kuma_ping(json: &Value) -> String {
    let Some(hb_list) = json.get("heartbeatList").and_then(|v| v.as_object()) else {
        return "—".to_string();
    };
    let mut total = 0u64;
    let mut count = 0u64;
    for beats in hb_list.values() {
        let Some(arr) = beats.as_array() else {
            continue;
        };
        let Some(latest) = arr.last() else {
            continue;
        };
        if let Some(ping) = latest.get("ping").and_then(|v| v.as_u64()) {
            total += ping;
            count += 1;
        }
    }
    if count == 0 {
        return "—".to_string();
    }
    format!("{} ms", total / count)
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
                        "avg_ping": parse_uptime_kuma_ping(&json),
                        "cert_expiring": json.get("incident").and_then(|v| v.as_array()).map(|a| a.len() as u64).unwrap_or(0),
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
            "avg_ping": "—",
            "cert_expiring": 0,
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
    let connector_version = result
        .get("connections")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("version").and_then(|v| v.as_str()))
        .unwrap_or("—");
    let origin_count = result
        .get("connections")
        .and_then(|v| v.as_array())
        .map(|arr| {
            let mut origins = std::collections::HashSet::new();
            for c in arr {
                if let Some(ip) = c.get("origin_ip").and_then(|v| v.as_str()) {
                    origins.insert(ip);
                }
            }
            origins.len() as u64
        })
        .unwrap_or(0);
    Some(json!({
        "type": "cloudflare_tunnel",
        "tunnel_name": name,
        "tunnel_status": status,
        "connections": connections,
        "colo_count": colo_count,
        "connector_version": connector_version,
        "origin_count": origin_count,
    }))
}

pub(crate) fn parse_qbittorrent_creds(raw: &str) -> Option<(String, String)> {
    parse_user_pass_creds(raw)
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
    let paused = count_qbittorrent_paused(&torrents);
    Some(json!({
        "type": "qbittorrent",
        "download_speed": format_qbit_speed(dl_speed),
        "upload_speed": format_qbit_speed(ul_speed),
        "active_downloads": downloading,
        "seeding": seeding,
        "free_disk": free_disk,
        "total_torrents": total_torrents,
        "paused_torrents": paused,
    }))
}

async fn fetch_bazarr(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    if api_key.trim().is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let key = api_key.to_string();
    let (movies_resp, episodes_resp, status_resp, langs_resp, providers_resp) = tokio::join!(
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
        client
            .get(format!("{}/api/languages", base))
            .header("X-Api-Key", &key)
            .send(),
        client
            .get(format!("{}/api/providers", base))
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
    let language_count = if let Ok(resp) = langs_resp {
        if resp.status().is_success() {
            resp.json::<Value>()
                .await
                .ok()
                .and_then(|v| v.as_array().map(|a| a.len() as u64))
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    let provider_count = if let Ok(resp) = providers_resp {
        if resp.status().is_success() {
            resp.json::<Value>()
                .await
                .ok()
                .and_then(|v| v.as_array().map(|a| a.len() as u64))
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    Some(json!({
        "type": "bazarr",
        "missing_movies": bazarr_wanted_count(&movies),
        "missing_episodes": bazarr_wanted_count(&episodes),
        "version": version,
        "health": health,
        "language_count": language_count,
        "provider_count": provider_count,
    }))
}

fn parse_user_pass_creds(raw: &str) -> Option<(String, String)> {
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

fn basic_auth_header(user: &str, pass: &str) -> String {
    use base64::Engine;
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"))
    )
}

pub(crate) fn parse_sabnzbd_speed(raw: &str) -> u64 {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return 0;
    }
    if let Ok(n) = trimmed.parse::<u64>() {
        return n;
    }
    let lower = trimmed.to_lowercase();
    let mut num: f64 = lower
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .parse()
        .unwrap_or(0.0);
    if lower.contains("mb") {
        num *= 1_000_000.0;
    } else if lower.contains("kb") {
        num *= 1_000.0;
    }
    num as u64
}

pub(crate) fn count_jackett_failed(indexers: &Value) -> u64 {
    let Some(arr) = indexers.as_array() else {
        return 0;
    };
    arr.iter()
        .filter(|i| {
            i.get("configured")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                && i.get("status")
                    .and_then(|v| v.as_u64())
                    .map(|s| s >= 2)
                    .unwrap_or(false)
        })
        .count() as u64
}

pub(crate) fn audiobookshelf_library_stats(libraries: &Value) -> (u64, u64) {
    let Some(arr) = libraries.as_array() else {
        return (0, 0);
    };
    let mut items = 0u64;
    for lib in arr {
        if let Some(stats) = lib.get("stats") {
            items += stats
                .get("totalItems")
                .or_else(|| stats.get("totalitems"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }
    }
    (arr.len() as u64, items)
}

pub(crate) fn frigate_camera_counts(config: &Value, stats: &Value) -> (u64, u64) {
    let total = config
        .get("cameras")
        .and_then(|c| c.as_object())
        .map(|o| o.len() as u64)
        .unwrap_or(0);
    let online = stats
        .get("cameras")
        .and_then(|c| c.as_object())
        .map(|o| {
            o.values()
                .filter(|cam| {
                    cam.get("camera_fps")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                        > 0.0
                })
                .count() as u64
        })
        .unwrap_or(total);
    (online, total)
}

async fn fetch_sabnzbd(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let queue_url = format!("{base}/api?mode=queue&output=json&apikey={key}");
    let status_url = format!("{base}/api?mode=status&output=json&apikey={key}");
    let (queue_resp, status_resp) = tokio::join!(
        client.get(&queue_url).send(),
        client.get(&status_url).send()
    );
    let queue: Value = queue_resp.ok()?.json().await.ok()?;
    let status: Value = status_resp.ok()?.json().await.ok()?;
    let queue_size = queue
        .pointer("/queue/noofslots")
        .or_else(|| queue.pointer("/queue/slots"))
        .and_then(|v| v.as_u64().or_else(|| v.as_array().map(|a| a.len() as u64)))
        .unwrap_or(0);
    let speed = status
        .get("speed")
        .and_then(|v| v.as_str())
        .map(parse_sabnzbd_speed)
        .or_else(|| status.get("speed").and_then(|v| v.as_u64()))
        .unwrap_or(0);
    let free_disk = status
        .get("freediskspace")
        .and_then(|v| v.as_str())
        .map(parse_sabnzbd_speed)
        .or_else(|| status.get("freediskspace").and_then(|v| v.as_u64()))
        .map(format_bytes_short)
        .unwrap_or_else(|| "—".to_string());
    let paused = status
        .get("paused")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let version = status
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("—")
        .to_string();
    Some(json!({
        "type": "sabnzbd",
        "queue_size": queue_size,
        "download_speed": format_qbit_speed(speed),
        "free_disk": free_disk,
        "paused": if paused { "Yes" } else { "No" },
        "version": version,
        "status": if paused { "Paused" } else { "Active" },
    }))
}

async fn nzbget_rpc(
    client: &reqwest::Client,
    base_url: &str,
    user: &str,
    pass: &str,
    method: &str,
) -> Option<Value> {
    let url = format!("{}/jsonrpc", base_url.trim_end_matches('/'));
    let body = json!({ "method": method, "params": [], "id": 1 });
    let resp = client
        .post(&url)
        .header("Authorization", basic_auth_header(user, pass))
        .json(&body)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    json.get("result").cloned()
}

async fn fetch_nzbget(client: &reqwest::Client, base_url: &str, creds_raw: &str) -> Option<Value> {
    let (user, pass) = parse_user_pass_creds(creds_raw)?;
    let (status, groups) = tokio::join!(
        nzbget_rpc(client, base_url, &user, &pass, "status"),
        nzbget_rpc(client, base_url, &user, &pass, "listgroups"),
    );
    let status = status?;
    let queue_size = groups
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(0);
    let speed = status
        .get("DownloadRate")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .unwrap_or(0);
    let free_disk = status
        .get("FreeDiskSpace")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
        .map(format_bytes_short)
        .unwrap_or_else(|| "—".to_string());
    let paused = status
        .get("DownloadPaused")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let version = status
        .get("Version")
        .and_then(|v| v.as_str())
        .unwrap_or("—")
        .to_string();
    Some(json!({
        "type": "nzbget",
        "queue_size": queue_size,
        "download_speed": format_qbit_speed(speed),
        "free_disk": free_disk,
        "paused": if paused { "Yes" } else { "No" },
        "version": version,
        "status": if paused { "Paused" } else { "Active" },
    }))
}

async fn transmission_rpc(
    client: &reqwest::Client,
    base_url: &str,
    session_id: Option<&str>,
    body: Value,
    auth: Option<(&str, &str)>,
) -> Option<(Value, Option<String>)> {
    let url = format!("{}/transmission/rpc", base_url.trim_end_matches('/'));
    let mut req = client.post(&url).json(&body);
    if let Some(sid) = session_id {
        req = req.header("X-Transmission-Session-Id", sid);
    }
    if let Some((user, pass)) = auth {
        req = req.header("Authorization", basic_auth_header(user, pass));
    }
    let resp = req.send().await.ok()?;
    if resp.status() == reqwest::StatusCode::CONFLICT {
        let sid = resp
            .headers()
            .get("x-transmission-session-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        if let Some(ref sid) = sid {
            return Box::pin(transmission_rpc(
                client,
                base_url,
                Some(sid.as_str()),
                body,
                auth,
            ))
            .await;
        }
        return None;
    }
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    Some((json, session_id.map(|s| s.to_string())))
}

pub(crate) fn count_transmission_torrents(args: &Value) -> (u64, u64, u64, u64) {
    let Some(torrents) = args.get("torrents").and_then(|v| v.as_array()) else {
        return (0, 0, 0, 0);
    };
    let total = torrents.len() as u64;
    let mut downloading = 0u64;
    let mut seeding = 0u64;
    let mut paused = 0u64;
    for t in torrents {
        let status = t.get("status").and_then(|v| v.as_u64()).unwrap_or(0);
        match status {
            0 => paused += 1,
            4 => downloading += 1,
            6 => seeding += 1,
            _ => {}
        }
    }
    (total, downloading, seeding, paused)
}

async fn fetch_transmission(
    accept_invalid_certs: bool,
    base_url: &str,
    creds_raw: &str,
) -> Option<Value> {
    let auth = parse_user_pass_creds(creds_raw);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .ok()?;
    let auth_pair = auth.as_ref().map(|(u, p)| (u.as_str(), p.as_str()));
    let session_body = json!({ "method": "session-get", "arguments": {} });
    let (session_json, sid) =
        transmission_rpc(&client, base_url, None, session_body, auth_pair).await?;
    let sid = sid.as_deref();
    let free_disk = session_json
        .pointer("/arguments/download-dir-free-space")
        .and_then(|v| v.as_u64())
        .map(format_bytes_short)
        .unwrap_or_else(|| "—".to_string());
    let torrent_body = json!({
        "method": "torrent-get",
        "arguments": { "fields": ["status", "rateDownload", "rateUpload"] }
    });
    let (torrent_json, _) =
        transmission_rpc(&client, base_url, sid, torrent_body, auth_pair).await?;
    let args = torrent_json.get("arguments")?;
    let (total, downloading, seeding, paused) = count_transmission_torrents(args);
    let dl_speed = args
        .get("torrents")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("rateDownload").and_then(|v| v.as_u64()))
                .sum()
        })
        .unwrap_or(0);
    let ul_speed = args
        .get("torrents")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("rateUpload").and_then(|v| v.as_u64()))
                .sum()
        })
        .unwrap_or(0);
    Some(json!({
        "type": "transmission",
        "download_speed": format_qbit_speed(dl_speed),
        "upload_speed": format_qbit_speed(ul_speed),
        "active_downloads": downloading,
        "seeding": seeding,
        "free_disk": free_disk,
        "total_torrents": total,
        "paused_torrents": paused,
    }))
}

async fn fetch_jackett(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let indexers_url = format!("{base}/api/v2.0/indexers?configured=true");
    let version_url = format!("{base}/api/v2.0/server/version");
    let (indexers_resp, version_resp) = tokio::join!(
        client.get(&indexers_url).header("X-Api-Key", key).send(),
        client.get(&version_url).header("X-Api-Key", key).send(),
    );
    let indexers: Value = indexers_resp.ok()?.json().await.ok()?;
    let version = if let Ok(resp) = version_resp {
        if resp.status().is_success() {
            resp.json::<Value>()
                .await
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "—".to_string())
        } else {
            "—".to_string()
        }
    } else {
        "—".to_string()
    };
    let total = indexers.as_array().map(|a| a.len() as u64).unwrap_or(0);
    let failed = count_jackett_failed(&indexers);
    Some(json!({
        "type": "jackett",
        "indexers_total": total,
        "failed_indexers": failed,
        "indexers_enabled": total.saturating_sub(failed),
        "version": version,
        "health": if failed == 0 { "Healthy" } else { "Issues" },
    }))
}

async fn tautulli_api(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    cmd: &str,
) -> Option<Value> {
    let url = format!(
        "{}/api/v2?apikey={}&cmd={}",
        base_url.trim_end_matches('/'),
        api_key.trim(),
        cmd
    );
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    json.pointer("/response/data").cloned()
}

async fn fetch_tautulli(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let (activity, libraries) = tokio::join!(
        tautulli_api(client, base_url, key, "get_activity"),
        tautulli_api(client, base_url, key, "get_libraries"),
    );
    let activity = activity?;
    let stream_count = activity
        .get("stream_count")
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .unwrap_or(0);
    let bandwidth = activity
        .get("total_bandwidth")
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .map(format_qbit_speed)
        .unwrap_or_else(|| "—".to_string());
    let library_count = libraries
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(0);
    Some(json!({
        "type": "tautulli",
        "stream_count": stream_count,
        "bandwidth": bandwidth,
        "library_count": library_count,
        "sessions": stream_count,
        "status": if stream_count > 0 { "Streaming" } else { "Idle" },
    }))
}

async fn fetch_audiobookshelf(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Option<Value> {
    let token = api_key.trim();
    if token.is_empty() {
        return None;
    }
    let url = format!("{}/api/libraries", base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let libraries: Value = resp.json().await.ok()?;
    let (library_count, item_count) = audiobookshelf_library_stats(&libraries);
    Some(json!({
        "type": "audiobookshelf",
        "library_count": library_count,
        "item_count": item_count,
        "libraries": library_count,
        "items": item_count,
        "status": if library_count > 0 { "Online" } else { "Empty" },
    }))
}

async fn fetch_immich(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let stats_url = format!("{base}/api/server/statistics");
    let about_url = format!("{base}/api/server/about");
    let (stats_resp, about_resp) = tokio::join!(
        client.get(&stats_url).header("x-api-key", key).send(),
        client.get(&about_url).header("x-api-key", key).send(),
    );
    let stats: Value = stats_resp.ok()?.json().await.ok()?;
    let version = if let Ok(resp) = about_resp {
        if resp.status().is_success() {
            resp.json::<Value>()
                .await
                .ok()
                .and_then(|v| {
                    v.get("version")
                        .and_then(|x| x.as_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| "—".to_string())
        } else {
            "—".to_string()
        }
    } else {
        "—".to_string()
    };
    let photos = stats.get("photos").and_then(|v| v.as_u64()).unwrap_or(0);
    let videos = stats.get("videos").and_then(|v| v.as_u64()).unwrap_or(0);
    let usage = stats
        .get("usage")
        .and_then(|v| v.as_u64())
        .map(format_bytes_short)
        .unwrap_or_else(|| "—".to_string());
    Some(json!({
        "type": "immich",
        "photos": photos,
        "videos": videos,
        "storage_used": usage,
        "version": version,
        "assets": photos + videos,
        "status": "Online",
    }))
}

async fn fetch_tdarr(client: &reqwest::Client, base_url: &str, _api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let status_url = format!("{base}/api/v2/status");
    let staged_body = json!({
        "data": { "collection": "StagedJSONDB", "mode": "getAll" }
    });
    let (status_resp, staged_resp) = tokio::join!(
        client.get(&status_url).send(),
        client
            .post(format!("{base}/api/v2/cruddb"))
            .json(&staged_body)
            .send(),
    );
    let status: Value = status_resp.ok()?.json().await.ok()?;
    let queue_size = if let Ok(resp) = staged_resp {
        if resp.status().is_success() {
            resp.json::<Value>()
                .await
                .ok()
                .and_then(|v| v.as_array().map(|a| a.len() as u64))
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    let workers = status
        .get("workerLimits")
        .and_then(|v| v.as_object())
        .map(|o| o.len() as u64)
        .or_else(|| {
            status
                .get("processes")
                .and_then(|v| v.as_object())
                .map(|o| o.len() as u64)
        })
        .unwrap_or(0);
    let health = if status.get("ok").and_then(|v| v.as_bool()) == Some(false) {
        "Unhealthy"
    } else {
        "Healthy"
    };
    Some(json!({
        "type": "tdarr",
        "queue_size": queue_size,
        "workers": workers,
        "health": health,
        "staged": queue_size,
        "status": health,
    }))
}

async fn fetch_maintainerr(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Option<Value> {
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let stats_url = format!("{base}/api/stats");
    let resp = client
        .get(&stats_url)
        .header("X-Api-Key", key)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let stats: Value = resp.json().await.ok()?;
    let issue_count = stats
        .get("totalIssueCount")
        .or_else(|| stats.get("issueCount"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let user_count = stats
        .get("totalUserCount")
        .or_else(|| stats.get("userCount"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let rule_count = stats
        .get("totalRuleCount")
        .or_else(|| stats.get("ruleCount"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    Some(json!({
        "type": "maintainerr",
        "issue_count": issue_count,
        "user_count": user_count,
        "rule_count": rule_count,
        "issues": issue_count,
        "rules": rule_count,
        "status": if issue_count > 0 { "Issues" } else { "Clear" },
    }))
}

async fn fetch_frigate(client: &reqwest::Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let config_url = format!("{base}/api/config");
    let stats_url = format!("{base}/api/stats");
    let mut config_req = client.get(&config_url);
    let mut stats_req = client.get(&stats_url);
    let key = api_key.trim();
    if !key.is_empty() {
        config_req = config_req.header("Authorization", format!("Bearer {key}"));
        stats_req = stats_req.header("Authorization", format!("Bearer {key}"));
    }
    let (config_resp, stats_resp) = tokio::join!(config_req.send(), stats_req.send());
    let config: Value = config_resp.ok()?.json().await.ok()?;
    let stats: Value = stats_resp.ok()?.json().await.ok()?;
    let (cameras_up, cameras_total) = frigate_camera_counts(&config, &stats);
    let detection_fps = stats
        .get("detectors")
        .and_then(|d| d.as_object())
        .map(|o| {
            o.values()
                .filter_map(|det| det.get("detection_fps").and_then(|v| v.as_f64()))
                .sum::<f64>()
        })
        .unwrap_or(0.0);
    Some(json!({
        "type": "frigate",
        "cameras_up": cameras_up,
        "cameras_total": cameras_total,
        "detection_fps": format!("{:.1}", detection_fps),
        "cameras": cameras_total,
        "online": cameras_up,
        "status": if cameras_up == cameras_total { "Online" } else { "Degraded" },
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
            let input_voltage = json
                .pointer("/ups/input.voltage")
                .or_else(|| json.get("input.voltage"))
                .and_then(|v| {
                    v.as_str()
                        .map(String::from)
                        .or_else(|| v.as_f64().map(|n| format!("{:.0}V", n)))
                })
                .unwrap_or_else(|| "—".to_string());
            let output_power = json
                .pointer("/ups/ups.power")
                .or_else(|| json.get("ups.power"))
                .and_then(|v| {
                    v.as_str()
                        .map(String::from)
                        .or_else(|| v.as_i64().map(|n| format!("{}W", n)))
                })
                .unwrap_or_else(|| "—".to_string());
            return Some(json!({
                "type": "peanut",
                "battery_percent": battery,
                "ups_load": load,
                "battery_runtime": runtime,
                "ups_status": status,
                "input_voltage": input_voltage,
                "output_power": output_power,
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
    fn arr_queue_size_reads_records_array() {
        let queue: Value = serde_json::json!({ "records": [{}, {}] });
        assert_eq!(super::arr_queue_size(&Some(queue)), 2);
        assert_eq!(super::arr_queue_size(&None), 0);
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

    #[test]
    fn parse_sabnzbd_speed_parses_human_and_numeric() {
        assert_eq!(parse_sabnzbd_speed("1.5 MB/s"), 1_500_000);
        assert_eq!(parse_sabnzbd_speed("500"), 500);
        assert_eq!(parse_sabnzbd_speed(""), 0);
    }

    #[test]
    fn count_jackett_failed_indexers_detects_status() {
        let json: Value = serde_json::json!([
            { "configured": true, "status": 2 },
            { "configured": true, "status": 1 },
            { "configured": false, "status": 2 }
        ]);
        assert_eq!(count_jackett_failed(&json), 1);
    }

    #[test]
    fn audiobookshelf_library_stats_sums_items() {
        let json: Value = serde_json::json!([
            { "stats": { "totalItems": 10 } },
            { "stats": { "totalItems": 5 } }
        ]);
        assert_eq!(audiobookshelf_library_stats(&json), (2, 15));
    }

    #[test]
    fn count_transmission_torrents_groups_by_status() {
        let json: Value = serde_json::json!({
            "torrents": [
                { "status": 4 },
                { "status": 6 },
                { "status": 0 }
            ]
        });
        assert_eq!(count_transmission_torrents(&json), (3, 1, 1, 1));
    }

    #[test]
    fn frigate_camera_counts_reads_config_and_stats() {
        let config: Value = serde_json::json!({
            "cameras": { "front": {}, "back": {} }
        });
        let stats: Value = serde_json::json!({
            "cameras": {
                "front": { "camera_fps": 5.0 },
                "back": { "camera_fps": 0.0 }
            }
        });
        assert_eq!(frigate_camera_counts(&config, &stats), (1, 2));
    }
}
