use crate::models::Session;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use getrandom;
use hex;
use rand_core::OsRng;
use rusqlite::{params, Connection};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;

pub(crate) fn revoke_sessions_for_user(
    sessions: &RwLock<HashMap<String, Session>>,
    username: &str,
) {
    let target = username.trim().to_lowercase();
    if target.is_empty() {
        return;
    }
    sessions
        .write()
        .unwrap()
        .retain(|_, session| session.username.to_lowercase() != target);
}

pub(crate) fn generate_agent_secret() -> String {
    generate_session_token()
}

pub async fn security_headers(req: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(HeaderName::from_static("x-frame-options"), HeaderValue::from_static("DENY"));
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: blob: http: https:; connect-src 'self' ws: wss: http: https:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
        ),
    );
    response
}

pub(crate) fn resolve_agent_secret(conn: &Connection) -> String {
    if let Ok(from_env) = std::env::var("AMUD_AGENT_SECRET") {
        if !from_env.is_empty() {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('agent_shared_secret', ?)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![from_env],
            )
            .ok();
            return from_env;
        }
    }

    let existing: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'agent_shared_secret'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    if !existing.is_empty() {
        return existing;
    }

    let secret = generate_agent_secret();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('agent_shared_secret', ?)",
        params![secret],
    )
    .ok();
    eprintln!(
        "AMUD SECURITY: Generated agent IPC secret. Set AMUD_AGENT_SECRET in the server and host agent systemd units."
    );
    secret
}

#[derive(Deserialize)]
pub(crate) struct AgentAuthMsg {
    auth: Option<String>,
}

pub(crate) fn agent_authenticated(agent_secret: &str, line: &str) -> bool {
    if agent_secret.is_empty() {
        return false;
    }
    serde_json::from_str::<AgentAuthMsg>(line)
        .ok()
        .and_then(|msg| msg.auth)
        .map(|token| {
            let expected = Sha256::digest(agent_secret.as_bytes());
            let actual = Sha256::digest(token.as_bytes());
            actual.as_slice().ct_eq(expected.as_slice()).into()
        })
        .unwrap_or(false)
}

// Password hashing helper
pub(crate) fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .unwrap_or_else(|_| legacy_sha256_password(password))
}

fn legacy_sha256_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

pub(crate) fn verify_password(stored_hash: &str, password: &str) -> (bool, bool) {
    if stored_hash.starts_with("$argon2") {
        let verified = PasswordHash::new(stored_hash)
            .ok()
            .and_then(|parsed| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed)
                    .ok()
            })
            .is_some();
        return (verified, false);
    }

    let legacy_hash = legacy_sha256_password(password);
    let verified = stored_hash.len() == legacy_hash.len()
        && stored_hash
            .as_bytes()
            .ct_eq(legacy_hash.as_bytes())
            .into();
    (verified, verified)
}

pub(crate) fn now_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub(crate) fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    if getrandom::getrandom(&mut bytes).is_ok() {
        URL_SAFE_NO_PAD.encode(bytes)
    } else {
        let seed = format!("{}:{:?}", now_epoch_secs(), Instant::now());
        hex::encode(Sha256::digest(seed.as_bytes()))
    }
}

pub(crate) fn session_cookie(token: &str) -> String {
    let secure = if std::env::var("AMUD_SECURE_COOKIES").ok().as_deref() == Some("1") {
        "; Secure"
    } else {
        ""
    };
    format!(
        "amud_session={}; Path=/; Max-Age=86400; HttpOnly; SameSite=Strict{}",
        token, secure
    )
}

pub(crate) fn expired_session_cookie() -> String {
    let secure = if std::env::var("AMUD_SECURE_COOKIES").ok().as_deref() == Some("1") {
        "; Secure"
    } else {
        ""
    };
    format!("amud_session=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict{}", secure)
}

pub(crate) fn login_rate_limited(login_attempts: &Mutex<HashMap<String, Vec<Instant>>>, username: &str) -> bool {
    const MAX_ATTEMPTS: usize = 5;
    const WINDOW: Duration = Duration::from_secs(5 * 60);
    const MAX_KEYS: usize = 2048;

    let key = username.trim().to_lowercase();
    let now = Instant::now();
    let mut attempts = login_attempts.lock().unwrap();
    attempts.retain(|_, values| {
        values.retain(|t| now.duration_since(*t) <= WINDOW);
        !values.is_empty()
    });
    if !attempts.contains_key(&key) && attempts.len() >= MAX_KEYS {
        return true;
    }
    attempts.get(&key).map(|v| v.len() >= MAX_ATTEMPTS).unwrap_or(false)
}

pub(crate) fn record_failed_login(login_attempts: &Mutex<HashMap<String, Vec<Instant>>>, username: &str) {
    let key = username.trim().to_lowercase();
    login_attempts
        .lock()
        .unwrap()
        .entry(key)
        .or_default()
        .push(Instant::now());
}

pub(crate) fn clear_failed_logins(login_attempts: &Mutex<HashMap<String, Vec<Instant>>>, username: &str) {
    login_attempts
        .lock()
        .unwrap()
        .remove(&username.trim().to_lowercase());
}

pub(crate) fn start_session_cleanup(sessions: Arc<RwLock<HashMap<String, Session>>>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 60)).await;
            let now = now_epoch_secs();
            sessions
                .write()
                .unwrap()
                .retain(|_, session| session.expires_at_epoch > now);
        }
    });
}

pub(crate) fn get_session(
    headers: &HeaderMap,
    sessions: &RwLock<HashMap<String, Session>>,
) -> Option<Session> {
    let token = headers
        .get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .map(|s| s.trim())
                .find(|s| s.starts_with("amud_session="))
                .map(|s| s["amud_session=".len()..].to_string())
        })?;

    let mut guard = sessions.write().unwrap();
    let session = guard.get(&token).cloned()?;
    if session.expires_at_epoch <= now_epoch_secs() {
        guard.remove(&token);
        None
    } else {
        Some(session)
    }
}

pub(crate) fn session_cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .map(|s| s.trim())
                .find(|s| s.starts_with("amud_session="))
                .map(|s| s["amud_session=".len()..].to_string())
        })
}

pub(crate) fn csrf_token_for_session(headers: &HeaderMap, sessions: &RwLock<HashMap<String, Session>>) -> String {
    let Some(cookie_token) = session_cookie_token(headers) else {
        return String::new();
    };
    sessions
        .read()
        .unwrap()
        .get(&cookie_token)
        .map(|s| s.csrf_token.clone())
        .unwrap_or_default()
}

pub(crate) fn validate_csrf(
    headers: &HeaderMap,
    sessions: &RwLock<HashMap<String, Session>>,
    form: Option<&HashMap<String, String>>,
) -> bool {
    let provided = form
        .and_then(|f| f.get("csrf_token").map(|s| s.as_str()))
        .or_else(|| headers.get("x-csrf-token").and_then(|v| v.to_str().ok()))
        .filter(|t| !t.is_empty());
    let Some(provided) = provided else {
        return false;
    };
    let Some(cookie_token) = session_cookie_token(headers) else {
        return false;
    };
    sessions
        .read()
        .unwrap()
        .get(&cookie_token)
        .map(|s| s.csrf_token == provided)
        .unwrap_or(false)
}

pub(crate) fn csrf_forbidden_response() -> Response {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"error":"Invalid CSRF token"}"#))
        .unwrap()
}
