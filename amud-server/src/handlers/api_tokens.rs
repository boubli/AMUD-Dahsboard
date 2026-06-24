use super::imports::*;
use crate::db::hash_api_token;
use rusqlite::Connection;
use std::sync::Mutex;

pub(crate) fn api_token_authorized(
    headers: &HeaderMap,
    state: &Arc<AppState>,
    required_scope: &str,
) -> bool {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if token.is_empty() {
        return false;
    }
    let hash = hash_api_token(token);
    let db = state.db.clone();
    let required = required_scope.to_string();
    let ok = with_db_blocking(db, move |conn| {
        if let Some((_id, scopes, expires_at)) = crate::db::load_api_token_by_hash(conn, &hash) {
            if let Some(exp) = expires_at {
                if !exp.is_empty() {
                    if let Ok(parsed) =
                        chrono::NaiveDateTime::parse_from_str(&exp, "%Y-%m-%d %H:%M:%S")
                    {
                        if parsed < chrono::Utc::now().naive_utc() {
                            return false;
                        }
                    }
                }
            }
            return scopes.split(',').any(|s| s.trim() == required.as_str());
        }
        false
    });
    ok
}

fn with_db_blocking(db: Arc<Mutex<Connection>>, f: impl FnOnce(&Connection) -> bool) -> bool {
    f(&db.lock().unwrap())
}

pub async fn list_api_tokens_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }
    let tokens = with_db(state.db.clone(), |db| {
        let mut list = Vec::new();
        if let Ok(mut stmt) = db.prepare(
            "SELECT id, name, scopes, created_at, expires_at FROM api_tokens ORDER BY id DESC",
        ) {
            if let Ok(mut rows) = stmt.query([]) {
                while let Ok(Some(row)) = rows.next() {
                    if let Ok(item) = (|| -> rusqlite::Result<serde_json::Value> {
                        Ok(serde_json::json!({
                            "id": row.get::<_, i64>(0)?,
                            "name": row.get::<_, String>(1)?,
                            "scopes": row.get::<_, String>(2)?,
                            "created_at": row.get::<_, String>(3)?,
                            "expires_at": row.get::<_, Option<String>>(4)?,
                        }))
                    })() {
                        list.push(item);
                    }
                }
            }
        }
        list
    })
    .await;
    api_json(StatusCode::OK, serde_json::json!({ "tokens": tokens }))
}

pub async fn create_api_token_handler(
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
    let name = form.get("name").cloned().unwrap_or_default();
    let scopes = form
        .get("scopes")
        .cloned()
        .unwrap_or_else(|| "read:apps,read:status".to_string());
    if name.is_empty() {
        return Redirect::to("/admin/settings?tab=security").into_response();
    }
    let raw_token = crate::auth::generate_session_token();
    let token_hash = hash_api_token(&raw_token);
    with_db(state.db.clone(), move |db| {
        let _ = db.execute(
            "INSERT INTO api_tokens (name, token_hash, scopes) VALUES (?, ?, ?)",
            params![name, token_hash, scopes],
        );
    })
    .await;
    api_json(
        StatusCode::OK,
        serde_json::json!({ "token": raw_token, "message": "Copy this token now — it will not be shown again." }),
    )
    .into_response()
}

pub async fn delete_api_token_handler(
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
            let _ = db.execute("DELETE FROM api_tokens WHERE id = ?", params![id]);
        })
        .await;
    }
    Redirect::to("/admin/settings?tab=security").into_response()
}
