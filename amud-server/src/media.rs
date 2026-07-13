use crate::db::load_settings_snapshot;
use crate::models::MediaStream;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub(crate) fn default_media_streams() -> HashMap<String, MediaStream> {
    let mut streams = HashMap::new();
    streams.insert(
        "plex".to_string(),
        MediaStream {
            status: "NOT CONFIGURED".to_string(),
            active: false,
            title: "Add a token in Edit App -> Integration".to_string(),
            ..Default::default()
        },
    );
    streams.insert(
        "jellyfin".to_string(),
        MediaStream {
            status: "NOT CONFIGURED".to_string(),
            active: false,
            title: "Add an API key in Edit App -> Integration".to_string(),
            ..Default::default()
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
                ..Default::default()
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
            ..Default::default()
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

    let session_id = first
        .get("Id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let is_paused = first
        .get("PlayState")
        .and_then(|v| v.get("IsPaused"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let poster = jellyfin_poster_path(item);

    MediaStream {
        status: "RUNNING".to_string(),
        active: true,
        title: media_summary(title, active.len()),
        current_time: format_media_time(current_ms),
        total_time: format_media_time(total_ms),
        progress_percent,
        poster,
        session_id,
        is_paused: Some(is_paused),
    }
}

/// Builds an internal poster proxy path from a Jellyfin NowPlayingItem, so the
/// browser never sees the Jellyfin URL or API key.
fn jellyfin_poster_path(item: &serde_json::Value) -> Option<String> {
    let item_id = item.get("Id").and_then(|v| v.as_str());
    let primary_tag = item
        .get("ImageTags")
        .and_then(|v| v.get("Primary"))
        .and_then(|v| v.as_str());
    let series_id = item.get("SeriesId").and_then(|v| v.as_str());
    let series_tag = item.get("SeriesPrimaryImageTag").and_then(|v| v.as_str());

    if let (Some(id), Some(tag)) = (item_id, primary_tag) {
        return Some(format!("/api/media/jellyfin/poster/{}?tag={}", id, tag));
    }
    if let (Some(id), Some(tag)) = (series_id, series_tag) {
        return Some(format!("/api/media/jellyfin/poster/{}?tag={}", id, tag));
    }
    None
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
                ..Default::default()
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
            ..Default::default()
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
        ..Default::default()
    }
}

/// Usable URL + API key from a per-app media integration, or None when the
/// app is missing either field.
pub(crate) fn app_media_credentials(app: &crate::models::App) -> Option<(String, String)> {
    let url = app.url.trim().trim_end_matches('/');
    let key = app.api_key.trim();
    if url.is_empty() || key.is_empty() {
        None
    } else {
        Some((url.to_string(), key.to_string()))
    }
}

pub(crate) fn start_media_poller(state: Arc<crate::models::AppState>) {
    tokio::spawn(async move {
        loop {
            if !crate::activity::is_active(&state) {
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }

            let cached = state.settings_cache.read().unwrap().clone();
            let settings = if cached.is_empty() {
                load_settings_snapshot(&state.db)
            } else {
                cached
            };
            let accept_invalid = settings
                .get("accept_invalid_certs")
                .map(|v| v == "1")
                .unwrap_or(false);
            let client =
                crate::http_client::select_http_client(&state.http_clients, accept_invalid).clone();
            // Credentials come from the per-app integration (Add App / Edit App),
            // not from global settings.
            let db_for_blocking = state.db.clone();
            let (jellyfin_app, plex_app) = tokio::task::spawn_blocking(move || {
                let db = db_for_blocking.lock().unwrap();
                (
                    crate::db::load_first_app_with_integration(&db, &["jellyfin", "emby"]),
                    crate::db::load_first_app_with_integration(&db, &["plex"]),
                )
            })
            .await
            .unwrap_or((None, None));
            let jellyfin_creds = jellyfin_app.as_ref().and_then(app_media_credentials);
            let plex_creds = plex_app.as_ref().and_then(app_media_credentials);
            let media_active = jellyfin_creds.is_some() || plex_creds.is_some();

            let jellyfin_fut = async {
                match &jellyfin_creds {
                    Some((url, key)) => Some(poll_jellyfin(&client, url, key).await),
                    None => jellyfin_app
                        .as_ref()
                        .map(|_| default_media_streams().remove("jellyfin").unwrap()),
                }
            };
            let plex_fut = async {
                match &plex_creds {
                    Some((url, key)) => Some(poll_plex(&client, url, key).await),
                    None => plex_app
                        .as_ref()
                        .map(|_| default_media_streams().remove("plex").unwrap()),
                }
            };
            let (jellyfin, plex) = tokio::join!(jellyfin_fut, plex_fut);

            {
                let mut streams = state.media_streams.write().unwrap();
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

            if !media_active {
                tokio::time::sleep(Duration::from_secs(300)).await;
                continue;
            }

            tokio::time::sleep(Duration::from_secs(crate::settings::setting_u64_bounded(
                &settings,
                "media_poll_interval_secs",
                5,
                3,
                120,
            )))
            .await;
        }
    });
}
