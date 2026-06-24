use super::imports::*;
use crate::models::ShareSession;
use axum::extract::Path;

pub async fn share_link_handler(
    Path(token): Path<String>,
    _headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let link = with_db(state.db.clone(), move |db| {
        crate::db::load_share_link_by_token(db, &token)
    })
    .await;

    let Some((allowed_paths, expires_at)) = link else {
        return Redirect::to("/").into_response();
    };

    if let Some(exp) = expires_at {
        if !exp.is_empty() {
            // Simple date compare — if parse fails, allow
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(&exp, "%Y-%m-%d %H:%M:%S") {
                if parsed < chrono::Utc::now().naive_utc() {
                    return Redirect::to("/login").into_response();
                }
            }
        }
    }

    let share_token = crate::auth::generate_session_token();
    let expires = now_epoch_secs() + 86400;
    state.share_sessions.write().unwrap().insert(
        share_token.clone(),
        ShareSession {
            allowed_paths,
            expires_at_epoch: expires,
        },
    );

    let mut response = Redirect::to("/").into_response();
    let cookie = format!("amud_share={share_token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400");
    response
        .headers_mut()
        .insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    response
}

pub async fn create_share_link_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }
    let label = form
        .get("label")
        .cloned()
        .unwrap_or_else(|| "Share link".to_string());
    let paths = form
        .get("allowed_paths")
        .cloned()
        .unwrap_or_else(|| "/".to_string());
    let token = crate::auth::generate_session_token();
    with_db(state.db.clone(), move |db| {
        let _ = db.execute(
            "INSERT INTO share_links (token, label, allowed_paths) VALUES (?, ?, ?)",
            params![token, label, paths],
        );
    })
    .await;
    Redirect::to("/admin/settings?tab=security").into_response()
}

pub async fn delete_share_link_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }
    if let Some(id) = form.get("id").and_then(|s| s.parse::<i64>().ok()) {
        with_db(state.db.clone(), move |db| {
            let _ = db.execute("DELETE FROM share_links WHERE id = ?", params![id]);
        })
        .await;
    }
    Redirect::to("/admin/settings?tab=security").into_response()
}

pub(crate) fn share_session_from_headers(
    headers: &HeaderMap,
    state: &AppState,
) -> Option<ShareSession> {
    let cookie = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("amud_share=") {
            let sessions = state.share_sessions.read().unwrap();
            if let Some(sess) = sessions.get(val) {
                if sess.expires_at_epoch > now_epoch_secs() {
                    return Some(sess.clone());
                }
            }
        }
    }
    None
}
