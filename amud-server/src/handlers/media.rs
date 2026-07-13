use super::imports::*;
use axum::extract::Query;

const MAX_POSTER_BYTES: usize = 3 * 1024 * 1024;

/// Jellyfin ids and image tags are hex strings (optionally dashed GUIDs).
fn valid_media_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// Resolve Jellyfin/Emby credentials from the per-app integration.
async fn jellyfin_credentials(state: &Arc<AppState>) -> Option<(String, String)> {
    let app = with_db(state.db.clone(), |db| {
        crate::db::load_first_app_with_integration(db, &["jellyfin", "emby"])
    })
    .await?;
    crate::media::app_media_credentials(&app)
}

fn media_http_client(state: &Arc<AppState>) -> reqwest::Client {
    let accept_invalid = {
        let cache = state.settings_cache.read().unwrap();
        cache
            .get("accept_invalid_certs")
            .map(|s| s == "1")
            .unwrap_or(false)
    };
    crate::http_client::select_http_client(&state.http_clients, accept_invalid).clone()
}

/// Proxies the primary poster image for a Jellyfin item so the browser never
/// needs the Jellyfin URL or API key.
pub async fn jellyfin_poster_handler(
    headers: HeaderMap,
    Path(item_id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "media_poster", 60, 60) {
        return resp;
    }
    if get_session(&headers, &state.sessions).is_none() {
        return api_json(
            StatusCode::UNAUTHORIZED,
            serde_json::json!({"error": "Unauthorized"}),
        );
    }
    if !valid_media_id(&item_id) {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Invalid item id"}),
        );
    }

    let Some((base_url, api_key)) = jellyfin_credentials(&state).await else {
        return api_json(
            StatusCode::NOT_FOUND,
            serde_json::json!({"error": "Jellyfin integration not configured"}),
        );
    };

    let mut url = format!(
        "{}/Items/{}/Images/Primary?maxHeight=360&quality=90",
        base_url, item_id
    );
    if let Some(tag) = query.get("tag").filter(|t| valid_media_id(t)) {
        url.push_str("&tag=");
        url.push_str(tag);
    }

    let client = media_http_client(&state);
    let resp = match client
        .get(&url)
        .header("X-Emby-Token", &api_key)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp,
        _ => {
            return api_json(
                StatusCode::NOT_FOUND,
                serde_json::json!({"error": "Poster unavailable"}),
            )
        }
    };

    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();
    if !content_type.starts_with("image/") {
        return api_json(
            StatusCode::NOT_FOUND,
            serde_json::json!({"error": "Poster unavailable"}),
        );
    }

    match resp.bytes().await {
        Ok(bytes) if !bytes.is_empty() && bytes.len() <= MAX_POSTER_BYTES => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "private, max-age=300")
            .body(Body::from(bytes))
            .unwrap(),
        _ => api_json(
            StatusCode::NOT_FOUND,
            serde_json::json!({"error": "Poster unavailable"}),
        ),
    }
}

/// Sends a playback command (stop / pause / unpause) to an active Jellyfin
/// session. Admin only.
pub async fn jellyfin_session_command_handler(
    headers: HeaderMap,
    Path((session_id, command)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return api_json(
            StatusCode::FORBIDDEN,
            serde_json::json!({"error": "Forbidden"}),
        );
    }
    if !validate_csrf(&headers, &state.sessions, None) {
        return csrf_forbidden_response();
    }
    if !valid_media_id(&session_id) {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Invalid session id"}),
        );
    }
    let jf_command = match command.as_str() {
        "stop" => "Stop",
        "pause" => "Pause",
        "unpause" => "Unpause",
        _ => {
            return api_json(
                StatusCode::BAD_REQUEST,
                serde_json::json!({"error": "Invalid command"}),
            )
        }
    };

    let Some((base_url, api_key)) = jellyfin_credentials(&state).await else {
        return api_json(
            StatusCode::NOT_FOUND,
            serde_json::json!({"error": "Jellyfin integration not configured"}),
        );
    };

    let url = format!(
        "{}/Sessions/{}/Playing/{}",
        base_url, session_id, jf_command
    );
    let client = media_http_client(&state);
    match client
        .post(&url)
        .header("X-Emby-Token", &api_key)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            api_json(StatusCode::OK, serde_json::json!({"success": true}))
        }
        Ok(resp) => api_json(
            StatusCode::BAD_GATEWAY,
            serde_json::json!({"error": format!("Jellyfin returned {}", resp.status())}),
        ),
        Err(e) => api_json(
            StatusCode::BAD_GATEWAY,
            serde_json::json!({"error": format!("Jellyfin unreachable: {}", e)}),
        ),
    }
}
