use super::api_tokens::api_token_authorized;
use super::imports::*;

pub async fn status_page_handler(
    Extension(csp): Extension<CspNonce>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    let settings = state.settings_cache.read().unwrap().clone();
    let status_public = settings
        .get("status_page_public")
        .map(|s| s.as_str())
        .unwrap_or("0")
        == "1";
    let is_admin = session.as_ref().map(|s| s.role == "Admin").unwrap_or(false);
    if !is_admin && !status_public && session.is_none() {
        return Redirect::to("/login").into_response();
    }

    let is_guest = !is_admin
        && (session.is_none() || session.as_ref().map(|s| s.role == "Guest").unwrap_or(false));
    let apps = with_db(state.db.clone(), load_apps_from_db).await;
    let mut visible_apps: Vec<_> = apps
        .into_iter()
        .filter(|a| a.integration_type != "rss")
        .collect();
    if is_guest {
        if let Some(allowed) = parse_guest_visible_categories(&settings) {
            visible_apps.retain(|app| {
                let cat = if app.category.is_empty() {
                    "General"
                } else {
                    app.category.as_str()
                };
                allowed.contains(cat)
            });
        }
        visible_apps.retain(|app| app.guest_visible);
    }

    let statuses = state.app_statuses.read().unwrap().clone();
    let branding = branding_from_settings(&settings);
    let mut rows = String::new();
    for app in &visible_apps {
        let st = statuses
            .get(&app.name)
            .map(|s| s.status.as_str())
            .unwrap_or("CHECKING");
        let latency = statuses
            .get(&app.name)
            .and_then(|s| s.latency_ms)
            .map(|ms| format!("{ms} ms"))
            .unwrap_or_else(|| "—".to_string());
        let status_class = match st {
            "ONLINE" => "status-online",
            "OFFLINE" => "status-offline",
            "BLOCKED" => "status-blocked",
            _ => "status-checking",
        };
        rows.push_str(&format!(
            r#"<tr><td>{}</td><td><span class="status-badge {}">{}</span></td><td>{}</td></tr>"#,
            escape_html(&app.name),
            status_class,
            escape_html(st),
            escape_html(&latency)
        ));
    }

    let ver = env!("CARGO_PKG_VERSION");
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="{}">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Status — {}</title>
<link rel="stylesheet" href="/static/style.css?v={ver}">
<style>:root {{ /* ROOT_CSS */ }}</style>
</head>
<body class="status-page">
<div class="dashboard-container" style="max-width:900px; margin:2rem auto;">
<header class="topbar" style="margin-bottom:1.5rem;">
<div class="brand-section"><h1 class="brand-title">Service Status</h1></div>
<a href="/" class="glass-panel" style="padding:0.5rem 1rem; text-decoration:none; color:inherit;">← Dashboard</a>
</header>
<div class="glass-panel" style="padding:1rem; overflow-x:auto;">
<table class="status-table" style="width:100%; border-collapse:collapse;">
<thead><tr><th style="text-align:left; padding:0.5rem;">Service</th><th style="text-align:left; padding:0.5rem;">Status</th><th style="text-align:left; padding:0.5rem;">Latency</th></tr></thead>
<tbody>{}</tbody>
</table>
</div>
<p style="font-size:0.78rem; color:var(--text-muted); margin-top:1rem;">Refreshes on page load. Live badges update on the main dashboard.</p>
</div>
</body></html>"#,
        escape_html(&branding.theme_mode),
        escape_html(&branding.app_name),
        rows
    );
    let html = html.replace("/* ROOT_CSS */", &build_root_css(&branding));
    Html(apply_csp_nonce(html, &csp.0)).into_response()
}

pub async fn api_status_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "api_status", 60, 60) {
        return resp.into_response();
    }
    let session = get_session(&headers, &state.sessions);
    if session.is_none() && !api_token_authorized(&headers, &state, "read:status") {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(r#"{"error":"Unauthorized"}"#))
            .unwrap()
            .into_response();
    }
    let statuses = state.app_statuses.read().unwrap().clone();
    api_json(StatusCode::OK, serde_json::json!({ "statuses": statuses })).into_response()
}
