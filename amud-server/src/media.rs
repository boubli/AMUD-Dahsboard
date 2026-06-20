use crate::apps::{is_jellyfin_app, is_plex_app};
use crate::db::{load_apps_from_db, load_settings_snapshot};
use crate::models::MediaStream;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

pub(crate) fn default_media_streams() -> HashMap<String, MediaStream> {
    let mut streams = HashMap::new();
    streams.insert(
        "plex".to_string(),
        MediaStream {
            status: "NOT CONFIGURED".to_string(),
            active: false,
            title: "Add Plex URL and token in Settings".to_string(),
            current_time: String::new(),
            total_time: String::new(),
            progress_percent: 0.0,
        },
    );
    streams.insert(
        "jellyfin".to_string(),
        MediaStream {
            status: "NOT CONFIGURED".to_string(),
            active: false,
            title: "Add Jellyfin URL and API key in Settings".to_string(),
            current_time: String::new(),
            total_time: String::new(),
            progress_percent: 0.0,
        },
    );
    streams
}

fn format_media_time(ms: i64) -> String {
    let total_seconds = (ms / 1000).max(0);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

fn media_summary(title: String, count: usize) -> String {
    if count > 1 {
        format!("{} (+{} more)", title, count - 1)
    } else {
        title
    }
}

pub(crate) async fn poll_jellyfin(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> MediaStream {
    if base_url.trim().is_empty() || api_key.trim().is_empty() {
        return default_media_streams().remove("jellyfin").unwrap();
    }

    let url = format!("{}/Sessions", base_url.trim_end_matches('/'));
    let resp = match client.get(url).header("X-Emby-Token", api_key).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return MediaStream {
                status: "ERROR".to_string(),
                active: false,
                title: format!("Jellyfin unreachable: {}", e),
                current_time: String::new(),
                total_time: String::new(),
                progress_percent: 0.0,
            }
        }
    };

    let sessions: Vec<serde_json::Value> = resp.json().await.unwrap_or_default();
    let active: Vec<&serde_json::Value> = sessions
        .iter()
        .filter(|session| session.get("NowPlayingItem").is_some())
        .collect();

    if active.is_empty() {
        return MediaStream {
            status: "RUNNING".to_string(),
            active: false,
            title: "No Active Streams".to_string(),
            current_time: String::new(),
            total_time: String::new(),
            progress_percent: 0.0,
        };
    }

    let first = active[0];
    let item = &first["NowPlayingItem"];
    let title = item
        .get("Name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Title")
        .to_string();
    let runtime_ticks = item
        .get("RunTimeTicks")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let position_ticks = first
        .get("PlayState")
        .and_then(|v| v.get("PositionTicks"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let total_ms = runtime_ticks / 10_000;
    let current_ms = position_ticks / 10_000;
    let progress_percent = if total_ms > 0 {
        (current_ms as f64 / total_ms as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    MediaStream {
        status: "RUNNING".to_string(),
        active: true,
        title: media_summary(title, active.len()),
        current_time: format_media_time(current_ms),
        total_time: format_media_time(total_ms),
        progress_percent,
    }
}

pub(crate) async fn poll_plex(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
) -> MediaStream {
    if base_url.trim().is_empty() || token.trim().is_empty() {
        return default_media_streams().remove("plex").unwrap();
    }

    let url = format!("{}/status/sessions", base_url.trim_end_matches('/'));
    let resp = match client
        .get(url)
        .header("X-Plex-Token", token)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return MediaStream {
                status: "ERROR".to_string(),
                active: false,
                title: format!("Plex unreachable: {}", e),
                current_time: String::new(),
                total_time: String::new(),
                progress_percent: 0.0,
            }
        }
    };

    let body: serde_json::Value = resp.json().await.unwrap_or_default();
    let sessions = body
        .pointer("/MediaContainer/Metadata")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if sessions.is_empty() {
        return MediaStream {
            status: "RUNNING".to_string(),
            active: false,
            title: "No Active Streams".to_string(),
            current_time: String::new(),
            total_time: String::new(),
            progress_percent: 0.0,
        };
    }

    let first = &sessions[0];
    let title = first
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Title")
        .to_string();
    let duration_ms = first.get("duration").and_then(|v| v.as_i64()).unwrap_or(0);
    let view_offset_ms = first
        .get("viewOffset")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let progress_percent = if duration_ms > 0 {
        (view_offset_ms as f64 / duration_ms as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    MediaStream {
        status: "RUNNING".to_string(),
        active: true,
        title: media_summary(title, sessions.len()),
        current_time: format_media_time(view_offset_ms),
        total_time: format_media_time(duration_ms),
        progress_percent,
    }
}

pub(crate) fn start_media_poller(
    db: Arc<Mutex<Connection>>,
    settings_cache: Arc<RwLock<HashMap<String, String>>>,
    media_streams: Arc<RwLock<HashMap<String, MediaStream>>>,
) {
    tokio::spawn(async move {
        loop {
            let cached = settings_cache.read().unwrap().clone();
            let settings = if cached.is_empty() {
                load_settings_snapshot(&db)
            } else {
                cached
            };
            let accept_invalid = settings.get("accept_invalid_certs").map(|v| v == "1").unwrap_or(false);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(4))
                .danger_accept_invalid_certs(accept_invalid)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            let db_for_blocking = db.clone();
            let apps = tokio::task::spawn_blocking(move || {
                let db = db_for_blocking.lock().unwrap();
                load_apps_from_db(&db)
            })
            .await
            .unwrap_or_default();
            let has_jellyfin = apps.iter().any(is_jellyfin_app);
            let has_plex = apps.iter().any(is_plex_app);

            let jellyfin_fut = async {
                if has_jellyfin {
                    Some(
                        poll_jellyfin(
                            &client,
                            settings
                                .get("jellyfin_url")
                                .map(|s| s.as_str())
                                .unwrap_or(""),
                            settings
                                .get("jellyfin_api_key")
                                .map(|s| s.as_str())
                                .unwrap_or(""),
                        )
                        .await,
                    )
                } else {
                    None::<MediaStream>
                }
            };
            let plex_fut = async {
                if has_plex {
                    Some(
                        poll_plex(
                            &client,
                            settings.get("plex_url").map(|s| s.as_str()).unwrap_or(""),
                            settings.get("plex_token").map(|s| s.as_str()).unwrap_or(""),
                        )
                        .await,
                    )
                } else {
                    None::<MediaStream>
                }
            };
            let (jellyfin, plex) = tokio::join!(jellyfin_fut, plex_fut);

            {
                let mut streams = media_streams.write().unwrap();
                if let Some(jellyfin) = jellyfin {
                    streams.insert("jellyfin".to_string(), jellyfin);
                } else {
                    streams.remove("jellyfin");
                    streams.remove("emby");
                }
                if let Some(plex) = plex {
                    streams.insert("plex".to_string(), plex);
                } else {
                    streams.remove("plex");
                }
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
