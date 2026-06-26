use super::imports::*;
use crate::rss_discover::discover_rss_feed;
use crate::security::{
    build_rss_outbound_client, favicon_fetch_url_for_host, get_rss_url_allowed,
    sanitize_favicon_host,
};
use axum::extract::Query;

#[derive(Deserialize)]
pub struct FaviconQuery {
    host: String,
}

/// POST /api/rss/discover — find RSS feed URL from a website homepage (admin only).
pub async fn rss_discover_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let site_url = form.get("site_url").cloned().unwrap_or_default();
    if site_url.trim().is_empty() {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({ "error": "Website URL is required" }),
        );
    }

    match discover_rss_feed(&site_url).await {
        Some(result) => api_json(StatusCode::OK, result),
        None => api_json(
            StatusCode::NOT_FOUND,
            serde_json::json!({
                "error": "No RSS or Atom feed found on that site. Try a feed generator or paste the XML URL directly."
            }),
        ),
    }
}

/// GET /api/rss/favicon?host=example.com — proxied favicon (avoids external DuckDuckGo dependency).
pub async fn rss_favicon_handler(Query(query): Query<FaviconQuery>) -> Response {
    let Some(host) = sanitize_favicon_host(&query.host) else {
        return Redirect::to("/static/feeds/icons/rss.svg").into_response();
    };

    let Some(target) = favicon_fetch_url_for_host(&host) else {
        return Redirect::to("/static/feeds/icons/rss.svg").into_response();
    };

    let Some(client) = build_rss_outbound_client("AMUD-Dashboard/1.5 Favicon-Proxy", 6) else {
        return Redirect::to("/static/feeds/icons/rss.svg").into_response();
    };

    let Some(resp) = get_rss_url_allowed(&client, &target).await else {
        return Redirect::to("/static/feeds/icons/rss.svg").into_response();
    };
    if !resp.status().is_success() {
        return Redirect::to("/static/feeds/icons/rss.svg").into_response();
    }
    let content_type = resp
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/x-icon")
        .to_string();
    let Ok(bytes) = resp.bytes().await else {
        return Redirect::to("/static/feeds/icons/rss.svg").into_response();
    };
    if bytes.is_empty() || bytes.len() > 512 * 1024 {
        return Redirect::to("/static/feeds/icons/rss.svg").into_response();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, content_type)
        .header(axum::http::header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(bytes))
        .unwrap_or_else(|_| Redirect::to("/static/feeds/icons/rss.svg").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_host_rejects_invalid() {
        assert!(sanitize_favicon_host("").is_none());
        assert!(sanitize_favicon_host("evil/host").is_none());
        assert_eq!(
            sanitize_favicon_host("www.Example.COM").as_deref(),
            Some("example.com")
        );
    }
}
