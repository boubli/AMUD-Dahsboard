use super::imports::*;
use axum::extract::Path;

pub async fn embed_app_handler(
    Extension(csp): Extension<CspNonce>,
    Path(app_id): Path<i64>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    let is_admin = session.as_ref().map(|s| s.role == "Admin").unwrap_or(false);
    let is_guest = !is_admin
        && (session.is_none() || session.as_ref().map(|s| s.role == "Guest").unwrap_or(false));

    let settings = state.settings_cache.read().unwrap().clone();
    if settings
        .get("iframe_embeds_enabled")
        .map(|s| s.as_str())
        .unwrap_or("0")
        != "1"
    {
        return Redirect::to("/").into_response();
    }

    let app = with_db(state.db.clone(), move |db| {
        crate::db::load_app_by_id(db, app_id)
    })
    .await;

    let Some(app) = app else {
        return Redirect::to("/").into_response();
    };

    if is_guest && !app.guest_visible {
        return Redirect::to("/").into_response();
    }
    if app.embed_mode != "iframe" && app.embed_mode != "tab" {
        return Redirect::to(&app.url).into_response();
    }

    let branding = branding_from_settings(&settings);
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="{}">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{} — {}</title>
<link rel="stylesheet" href="/static/style.css">
<style>:root {{ /* ROOT_CSS */ }} body {{ margin:0; }} .embed-frame {{ width:100vw; height:100vh; border:0; }}</style>
</head>
<body>
<iframe class="embed-frame" src="{}" sandbox="allow-scripts allow-same-origin allow-forms allow-popups" referrerpolicy="no-referrer" title="{}"></iframe>
</body></html>"#,
        escape_html(&branding.theme_mode),
        escape_html(&app.name),
        escape_html(&branding.app_name),
        escape_html(&app.url),
        escape_html(&app.name)
    );
    let html = html.replace("/* ROOT_CSS */", &build_root_css(&branding));
    Html(apply_csp_nonce(html, &csp.0)).into_response()
}
