use super::imports::*;
use axum::extract::Query;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use std::sync::RwLock;

static OIDC_STATES: once_cell::sync::Lazy<RwLock<Vec<String>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(Vec::new()));

pub async fn oidc_login_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let settings = state.settings_cache.read().unwrap().clone();
    if settings
        .get("oidc_enabled")
        .map(|s| s.as_str())
        .unwrap_or("0")
        != "1"
    {
        return Redirect::to("/login").into_response();
    }
    let client = match build_oauth_client(&settings).await {
        Ok(c) => c,
        Err(_) => return Redirect::to("/login?error=1").into_response(),
    };
    let (auth_url, csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();
    OIDC_STATES.write().unwrap().push(csrf.secret().clone());
    Redirect::to(auth_url.as_str()).into_response()
}

pub async fn oidc_callback_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let settings = state.settings_cache.read().unwrap().clone();
    if settings
        .get("oidc_enabled")
        .map(|s| s.as_str())
        .unwrap_or("0")
        != "1"
    {
        return Redirect::to("/login").into_response();
    }

    let state_param = params.get("state").cloned().unwrap_or_default();
    {
        let mut states = OIDC_STATES.write().unwrap();
        if let Some(pos) = states.iter().position(|s| s == &state_param) {
            states.remove(pos);
        } else {
            return Redirect::to("/login?error=1").into_response();
        }
    }

    let code = match params.get("code") {
        Some(c) => AuthorizationCode::new(c.clone()),
        None => return Redirect::to("/login?error=1").into_response(),
    };

    let client = match build_oauth_client(&settings).await {
        Ok(c) => c,
        Err(_) => return Redirect::to("/login?error=1").into_response(),
    };

    let token = match client
        .exchange_code(code)
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login?error=1").into_response(),
    };

    let access = token.access_token().secret();
    let issuer = settings.get("oidc_issuer").cloned().unwrap_or_default();
    let userinfo_url = format!("{}/userinfo", issuer.trim_end_matches('/'));
    let username = match reqwest::Client::new()
        .get(&userinfo_url)
        .bearer_auth(access)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            body.get("preferred_username")
                .or_else(|| body.get("email"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "oidc-user".to_string())
        }
        _ => "oidc-user".to_string(),
    };

    let default_role = settings
        .get("oidc_default_role")
        .map(|s| s.as_str())
        .unwrap_or("Guest");
    let role = if default_role == "Admin" {
        "Admin"
    } else {
        "Guest"
    };

    let username_db = username.clone();
    with_db(state.db.clone(), move |db| {
        let exists: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM users WHERE lower(username) = lower(?)",
                params![username_db],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists == 0 {
            let hash = hash_password(&generate_session_token());
            let _ = db.execute(
                "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
                params![username_db, hash, role],
            );
        }
    })
    .await;

    let session_token = generate_session_token();
    let csrf = generate_session_token();
    let expires = now_epoch_secs() + 86400;
    state.sessions.write().unwrap().insert(
        session_token.clone(),
        crate::models::Session {
            username,
            role: role.to_string(),
            expires_at_epoch: expires,
            csrf_token: csrf,
        },
    );

    let mut response = Redirect::to("/").into_response();
    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        session_cookie(&session_token).parse().unwrap(),
    );
    let _ = headers;
    response
}

async fn build_oauth_client(settings: &HashMap<String, String>) -> Result<BasicClient, String> {
    let issuer = settings.get("oidc_issuer").map(|s| s.trim()).unwrap_or("");
    let client_id = settings
        .get("oidc_client_id")
        .map(|s| s.trim())
        .unwrap_or("");
    let client_secret = settings
        .get("oidc_client_secret")
        .map(|s| s.trim())
        .unwrap_or("");
    let redirect = settings
        .get("oidc_redirect_uri")
        .map(|s| s.trim())
        .unwrap_or("");
    if issuer.is_empty() || client_id.is_empty() || redirect.is_empty() {
        return Err("missing config".to_string());
    }

    let discovery_url = format!(
        "{}/.well-known/openid-configuration",
        issuer.trim_end_matches('/')
    );
    let doc: serde_json::Value = reqwest::get(&discovery_url)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let auth_endpoint = doc
        .get("authorization_endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "no auth endpoint".to_string())?;
    let token_endpoint = doc
        .get("token_endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "no token endpoint".to_string())?;

    let client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        if client_secret.is_empty() {
            None
        } else {
            Some(ClientSecret::new(client_secret.to_string()))
        },
        AuthUrl::new(auth_endpoint.to_string()).map_err(|e| e.to_string())?,
        Some(TokenUrl::new(token_endpoint.to_string()).map_err(|e| e.to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect.to_string()).map_err(|e| e.to_string())?);
    Ok(client)
}
