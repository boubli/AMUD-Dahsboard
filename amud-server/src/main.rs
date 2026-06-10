use axum::{
    middleware::{self, Next},
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        Multipart, State,
    },
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand_core::OsRng;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path as FilePath;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use tokio::net::TcpListener as TokioTcpListener;
#[cfg(unix)]
use tokio::net::UnixListener as TokioUnixListener;

// Data models
#[derive(Clone, Serialize, Deserialize)]
struct App {
    id: i64,
    name: String,
    url: String,
    icon: String,
    description: String,
    category: String,
    node_tag: String,
}

#[derive(Clone, Serialize)]
struct Session {
    username: String,
    role: String,
    expires_at_epoch: u64,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct LxcContainer {
    vmid: i64,
    status: String,
    name: String,
    cpu: Option<f64>,
    maxmem: Option<i64>,
    mem: Option<i64>,
    maxdisk: Option<i64>,
    disk: Option<i64>,
    uptime: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct AgentTelemetry {
    cpu_usage: i32,
    ram_usage: i32,
    ram_used_gb: f64,
    ram_total_gb: f64,
    cpu_temp: f64,
    disk_usage: i32,
    disk_used_gb: f64,
    disk_total_gb: f64,
    #[serde(default)]
    lxc_containers: Vec<LxcContainer>,
    #[serde(default)]
    network: Option<NetworkTelemetry>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct NetworkTelemetry {
    internal_tx: String,
    internal_rx: String,
    external_tx: String,
    external_rx: String,
}

#[derive(Serialize, Clone)]
struct MediaStream {
    status: String,
    active: bool,
    title: String,
    current_time: String,
    total_time: String,
    progress_percent: f64,
}

#[derive(Serialize, Clone, Default)]
struct AppStatus {
    status: String,
    latency_ms: Option<u128>,
}

#[derive(Serialize, Clone)]
struct FullTelemetry {
    system: AgentTelemetry,
    network: NetworkTelemetry,
    streams: HashMap<String, MediaStream>,
    app_statuses: HashMap<String, AppStatus>,
}

#[derive(Clone, Serialize, Deserialize)]
struct PveTestResult {
    success: bool,
    error: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Webhook {
    id: i64,
    name: String,
    url: String,
    event_types: String,
    is_active: i32,
}

// Global App State
#[allow(dead_code)]
struct AppState {
    db: Arc<Mutex<Connection>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    latest_telemetry: Arc<RwLock<AgentTelemetry>>,
    agent_connected: Arc<RwLock<bool>>,
    media_streams: Arc<RwLock<HashMap<String, MediaStream>>>,
    app_statuses: Arc<RwLock<HashMap<String, AppStatus>>>,
    agent_command_tx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<String>>>>,
    pve_test_response: Arc<RwLock<Option<PveTestResult>>>,
    alert_cooldowns: Arc<Mutex<HashMap<String, std::time::Instant>>>,
    login_attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    agent_secret: Arc<String>,
}


// Global default settings
fn get_default_settings() -> HashMap<&'static str, &'static str> {
    let mut s = HashMap::new();
    s.insert("app_name", "AMUD");
    s.insert("tagline", "Homelab Operations Cockpit");
    s.insert("accent_color", "#cf6427");
    s.insert("custom_bg_url", "/static/wallpaper.png");
    s.insert("app_logo", "");
    s.insert("glass_blur_intensity", "16");
    s.insert("glass_opacity", "0.45");
    s.insert("bento_radius", "16");
    s.insert("grid_columns", "3");
    s.insert("jellyfin_url", "");
    s.insert("jellyfin_api_key", "");
    s.insert("plex_url", "");
    s.insert("plex_token", "");
    s.insert("pve_api_token", "");
    s.insert("donate_enabled", "1");

    s.into()
}

// Donation links are fixed to the AMUD author. Self-hosters can toggle the
// Support card on/off in Settings, but cannot change these links.
const DONATION_MESSAGE: &str = "AMUD is completely free and you already have every feature unlocked. A donation is not required and unlocks nothing extra - it is simply a kind way to support continued development. Thank you!";
const DONATION_LINKS: [(&str, &str, &str); 3] = [
    ("https://github.com/sponsors/boubli", "GitHub Sponsors", "github"),
    ("https://buy.stripe.com/cNi14n6b9a7v5Jg4Rq4ko00", "Donate via Card", "credit-card"),
    ("https://ko-fi.com/Youssefboubli", "Ko-fi", "coffee"),
];

#[tokio::main]
async fn main() {
    println!("AMUD Server starting up in Rust...");

    // Init DB
    fs::create_dir_all("data").ok();
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "data/amud.db".to_string());
    let conn = Connection::open(&db_path).expect("Failed to open SQLite database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            icon TEXT,
            description TEXT,
            category TEXT DEFAULT 'General',
            node_tag TEXT DEFAULT 'Local'
        );",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'Guest'
        );",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS webhooks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            event_types TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
        [],
    )
    .unwrap();


    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            color TEXT DEFAULT '#64748b',
            sort_order INTEGER DEFAULT 0
        );",
        [],
    )
    .unwrap();

    // Seed default categories if empty
    {
        let mut stmt_cats = conn.prepare("SELECT COUNT(*) FROM categories").unwrap();
        let cat_count: i64 = stmt_cats.query_row([], |r| r.get(0)).unwrap();
        if cat_count == 0 {
            println!("Seeding default categories...");
            let defaults = [
                ("General", "#64748b", 0),
                ("Media", "#f97316", 1),
                ("Infrastructure", "#3b82f6", 2),
                ("Monitoring", "#10b981", 3),
                ("Network", "#8b5cf6", 4),
                ("Storage", "#ec4899", 5),
            ];
            for (name, color, order) in defaults {
                conn.execute(
                    "INSERT INTO categories (name, color, sort_order) VALUES (?, ?, ?)",
                    params![name, color, order],
                )
                .ok();
            }
        }
    }

    // Seed default settings if they don't exist
    {
        println!("Ensuring default settings exist...");
        for (key, val) in get_default_settings() {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)",
                params![key, val],
            )
            .ok();
        }
    }

    // Check users count
    {
        let mut stmt_users = conn.prepare("SELECT COUNT(*) FROM users").unwrap();
        let user_count: i64 = stmt_users.query_row([], |r| r.get(0)).unwrap();
        if user_count == 0 {
            println!("Seeding security roles...");
            let admin_hash = hash_password("admin");
            let guest_hash = hash_password("guest");
            conn.execute(
                "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
                params!["admin", admin_hash, "Admin"],
            )
            .ok();
            conn.execute(
                "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
                params!["guest", guest_hash, "Guest"],
            )
            .ok();
        }
    }

    let agent_secret = resolve_agent_secret(&conn);

    let shared_db = Arc::new(Mutex::new(conn));
    let sessions = Arc::new(RwLock::new(HashMap::<String, Session>::new()));
    let latest_telemetry = Arc::new(RwLock::new(AgentTelemetry::default()));
    let agent_connected = Arc::new(RwLock::new(false));
    let media_streams = Arc::new(RwLock::new(default_media_streams()));
    let app_statuses = Arc::new(RwLock::new(HashMap::new()));
    let agent_command_tx = Arc::new(Mutex::new(None));
    let pve_test_response = Arc::new(RwLock::new(None));

    let state = Arc::new(AppState {
        db: shared_db.clone(),
        sessions: sessions.clone(),
        latest_telemetry: latest_telemetry.clone(),
        agent_connected: agent_connected.clone(),
        media_streams: media_streams.clone(),
        app_statuses: app_statuses.clone(),
        agent_command_tx: agent_command_tx.clone(),
        pve_test_response: pve_test_response.clone(),
        alert_cooldowns: Arc::new(Mutex::new(HashMap::new())),
        login_attempts: Arc::new(Mutex::new(HashMap::new())),
        agent_secret: Arc::new(agent_secret),
    });

    // Start Host Agent listener (Background task)
    start_agent_listener(state.clone());
    start_session_cleanup(sessions.clone());

    start_media_poller(shared_db.clone(), media_streams);
    start_status_poller(shared_db.clone(), app_statuses);

    // Set up Axum Router
    let app = Router::new()
        .route("/", get(dashboard_handler))
        .route("/login", get(login_page).post(login_handler))
        .route("/logout", get(logout_handler))
        .route("/ws", get(ws_handler))
        .route("/admin/settings", get(settings_page_handler).post(settings_handler))
        .route("/admin/proxmox/test", post(test_proxmox_handler))
        .route(
            "/admin/upload",
            post(upload_handler).layer(axum::extract::DefaultBodyLimit::max(8 * 1024 * 1024)),
        )
        .route("/admin/credentials", post(credentials_handler))
        .route("/apps", post(add_app_handler))
        .route("/apps/delete", post(delete_app_handler))
        .route("/apps/edit", post(edit_app_handler))
        .route("/apps/action", post(app_action_handler))
        .route(
            "/api/categories",
            get(list_categories_handler).post(add_category_handler),
        )
        .route("/api/categories/delete", post(delete_category_handler))
        .route("/api/categories/edit", post(edit_category_handler))
        .route(
            "/api/webhooks",
            get(list_webhooks_handler).post(add_webhook_handler),
        )
        .route("/api/webhooks/edit", post(edit_webhook_handler))
        .route("/api/webhooks/delete", post(delete_webhook_handler))
        .route("/api/webhooks/test", post(test_webhook_handler))
        .route(
            "/api/users",
            get(list_users_handler).post(add_user_handler),
        )
        .route("/api/users/edit", post(edit_user_handler))
        .route("/api/users/delete", post(delete_user_handler))

        .nest_service(
            "/uploads",
            tower_http::services::ServeDir::new("data/uploads"),
        )
        .nest_service("/static", tower_http::services::ServeDir::new("ui/static"))
        .layer(middleware::from_fn(security_headers))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("AMUD Web Server listening online on http://{}", addr);

    let listener = TokioTcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn generate_agent_secret() -> String {
    let seed = format!(
        "amud-agent-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    hex::encode(Sha256::digest(seed.as_bytes()))
}

async fn security_headers(req: Request<axum::body::Body>, next: Next) -> Response {
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

fn resolve_agent_secret(conn: &Connection) -> String {
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
struct AgentAuthMsg {
    auth: Option<String>,
}

fn agent_authenticated(agent_secret: &str, line: &str) -> bool {
    if agent_secret.is_empty() {
        return true;
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
fn hash_password(password: &str) -> String {
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

fn verify_password(stored_hash: &str, password: &str) -> (bool, bool) {
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

fn now_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    if getrandom::getrandom(&mut bytes).is_ok() {
        URL_SAFE_NO_PAD.encode(bytes)
    } else {
        let seed = format!("{}:{:?}", now_epoch_secs(), Instant::now());
        hex::encode(Sha256::digest(seed.as_bytes()))
    }
}

fn session_cookie(token: &str) -> String {
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

fn expired_session_cookie() -> String {
    let secure = if std::env::var("AMUD_SECURE_COOKIES").ok().as_deref() == Some("1") {
        "; Secure"
    } else {
        ""
    };
    format!("amud_session=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict{}", secure)
}

fn login_rate_limited(login_attempts: &Mutex<HashMap<String, Vec<Instant>>>, username: &str) -> bool {
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

fn record_failed_login(login_attempts: &Mutex<HashMap<String, Vec<Instant>>>, username: &str) {
    let key = username.trim().to_lowercase();
    login_attempts
        .lock()
        .unwrap()
        .entry(key)
        .or_default()
        .push(Instant::now());
}

fn clear_failed_logins(login_attempts: &Mutex<HashMap<String, Vec<Instant>>>, username: &str) {
    login_attempts
        .lock()
        .unwrap()
        .remove(&username.trim().to_lowercase());
}

fn start_session_cleanup(sessions: Arc<RwLock<HashMap<String, Session>>>) {
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

// Escape user-controlled text before injecting it into HTML.
fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

// Force a safe scheme on user-supplied URLs to neutralize javascript:/data: vectors.
fn normalize_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    }
}

// Get User session helper
fn get_session(
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

fn default_media_streams() -> HashMap<String, MediaStream> {
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
        "emby".to_string(),
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

fn load_settings_snapshot(db: &Arc<Mutex<Connection>>) -> HashMap<String, String> {
    let mut settings = HashMap::new();
    let db = db.lock().unwrap();
    if let Ok(mut stmt) = db.prepare("SELECT key, value FROM settings") {
        if let Ok(mut rows) = stmt.query([]) {
            while let Ok(Some(row)) = rows.next() {
                if let (Ok(key), Ok(value)) = (row.get::<_, String>(0), row.get::<_, String>(1)) {
                    settings.insert(key, value);
                }
            }
        }
    }
    settings
}

async fn poll_jellyfin(client: &reqwest::Client, base_url: &str, api_key: &str) -> MediaStream {
    if base_url.trim().is_empty() || api_key.trim().is_empty() {
        return default_media_streams().remove("emby").unwrap();
    }

    let url = format!(
        "{}/Sessions?api_key={}",
        base_url.trim_end_matches('/'),
        api_key
    );
    let resp = match client.get(url).send().await {
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

async fn poll_plex(client: &reqwest::Client, base_url: &str, token: &str) -> MediaStream {
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
    let view_offset_ms = first.get("viewOffset").and_then(|v| v.as_i64()).unwrap_or(0);
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

fn start_media_poller(
    db: Arc<Mutex<Connection>>,
    media_streams: Arc<RwLock<HashMap<String, MediaStream>>>,
) {
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        loop {
            let settings = load_settings_snapshot(&db);
            let jellyfin = poll_jellyfin(
                &client,
                settings.get("jellyfin_url").map(|s| s.as_str()).unwrap_or(""),
                settings
                    .get("jellyfin_api_key")
                    .map(|s| s.as_str())
                    .unwrap_or(""),
            );
            let plex = poll_plex(
                &client,
                settings.get("plex_url").map(|s| s.as_str()).unwrap_or(""),
                settings.get("plex_token").map(|s| s.as_str()).unwrap_or(""),
            );
            let (jellyfin, plex) = tokio::join!(jellyfin, plex);

            {
                let mut streams = media_streams.write().unwrap();
                streams.insert("emby".to_string(), jellyfin);
                streams.insert("plex".to_string(), plex);
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

fn start_status_poller(
    db: Arc<Mutex<Connection>>,
    app_statuses: Arc<RwLock<HashMap<String, AppStatus>>>,
) {
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        loop {
            let apps = {
                let db = db.lock().unwrap();
                let mut apps = Vec::<(String, String)>::new();
                if let Ok(mut stmt) = db.prepare("SELECT name, url FROM apps") {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    }) {
                        for app in rows.flatten() {
                            apps.push(app);
                        }
                    }
                }
                apps
            };

            let mut next = HashMap::new();
            for (name, url) in apps {
                let started = Instant::now();
                let status = match client.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                        AppStatus {
                            status: "ONLINE".to_string(),
                            latency_ms: Some(started.elapsed().as_millis()),
                        }
                    }
                    Ok(_) | Err(_) => AppStatus {
                        status: "OFFLINE".to_string(),
                        latency_ms: None,
                    },
                };
                next.insert(name.to_lowercase(), status);
            }

            *app_statuses.write().unwrap() = next;
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });
}

async fn send_webhook_notification(
    url: String,
    name: String,
    event_type: &str,
    container_name: &str,
    vmid: i64,
    status: &str,
    provider: &str,
) {
    let client = reqwest::Client::new();
    let is_discord = url.contains("discord.com/api/webhooks/");
    let is_telegram = url.contains("api.telegram.org/bot");

    let response = if is_discord {
        let title = if event_type == "test" {
            "ðŸ”” AMUD Webhook Test".to_string()
        } else if status == "running" {
            format!("ðŸŸ¢ Container Started: {}", container_name)
        } else {
            format!("ðŸ”´ Container Stopped: {}", container_name)
        };

        let desc = if event_type == "test" {
            "Your AMUD Webhook Alerts Engine is successfully configured and ready to notify!".to_string()
        } else {
            format!("Container **{}** is now **{}**.", container_name, status)
        };

        let color = if event_type == "test" {
            0x2ecc71
        } else if status == "running" {
            0x10b981
        } else {
            0xef4444
        };

        let mut fields = vec![];
        if event_type != "test" {
            fields.push(serde_json::json!({
                "name": "Provider",
                "value": provider,
                "inline": true
            }));
            fields.push(serde_json::json!({
                "name": "VMID / ID",
                "value": vmid.to_string(),
                "inline": true
            }));
        }

        let payload = serde_json::json!({
            "username": "AMUD Alerts",
            "embeds": [{
                "title": title,
                "description": desc,
                "color": color,
                "fields": fields,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }]
        });

        client.post(&url).json(&payload).send().await
    } else if is_telegram {
        let text = if event_type == "test" {
            "<b>ðŸ”” AMUD Alert Test</b>\nYour Webhook Alerts Engine is successfully configured and ready to notify!".to_string()
        } else {
            let status_emoji = if status == "running" { "ðŸŸ¢" } else { "ðŸ”´" };
            format!(
                "{} <b>AMUD Alert: Container Status Changed</b>\n\n<b>Container:</b> <code>{}</code>\n<b>Status:</b> <code>{}</code>\n<b>Provider:</b> <code>{}</code>\n<b>VMID/ID:</b> <code>{}</code>",
                status_emoji, container_name, status.to_uppercase(), provider, vmid
            )
        };

        let parsed_url = reqwest::Url::parse(&url).ok();
        let chat_id = parsed_url.and_then(|u| {
            u.query_pairs()
                .find(|(k, _)| k == "chat_id")
                .map(|(_, v)| v.into_owned())
        });

        let payload = if let Some(cid) = chat_id {
            serde_json::json!({
                "chat_id": cid,
                "text": text,
                "parse_mode": "HTML"
            })
        } else {
            serde_json::json!({
                "text": text,
                "parse_mode": "HTML"
            })
        };

        client.post(&url).json(&payload).send().await
    } else {
        let payload = serde_json::json!({
            "event": event_type,
            "container": {
                "name": container_name,
                "vmid": vmid,
                "status": status,
                "provider": provider
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        client.post(&url).json(&payload).send().await
    };

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("Webhook '{}' successfully sent notification for '{}'", name, container_name);
            } else {
                eprintln!(
                    "Webhook '{}' failed with status code: {}. Body: {:?}",
                    name,
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to send webhook '{}': {}", name, e);
        }
    }
}

fn check_container_alerts(
    old_telemetry: &AgentTelemetry,
    new_telemetry: &AgentTelemetry,
    state: &Arc<AppState>,
) {
    let old_containers = &old_telemetry.lxc_containers;
    let new_containers = &new_telemetry.lxc_containers;

    let old_map: HashMap<i64, &LxcContainer> = old_containers
        .iter()
        .map(|c| (c.vmid, c))
        .collect();

    for new_c in new_containers {
        if let Some(old_c) = old_map.get(&new_c.vmid) {
            let old_status = &old_c.status;
            let new_status = &new_c.status;

            if old_status != new_status {
                let is_running_now = new_status == "running";
                let event_type = if is_running_now { "container_started" } else { "container_stopped" };
                let cooldown_key = format!("{}:{}", if new_c.vmid < 0 { "docker" } else { "lxc" }, new_c.name);

                {
                    let mut cooldowns = state.alert_cooldowns.lock().unwrap();
                    if let Some(&last_alert) = cooldowns.get(&cooldown_key) {
                        if last_alert.elapsed() < Duration::from_secs(60) {
                            println!("Alert for {} is suppressed due to cooldown", cooldown_key);
                            continue;
                        }
                    }
                    cooldowns.insert(cooldown_key.clone(), std::time::Instant::now());
                }

                let webhooks = {
                    let db = state.db.lock().unwrap();
                    let mut stmt = db.prepare("SELECT id, name, url, event_types, is_active FROM webhooks WHERE is_active = 1").unwrap();
                    let mut rows = stmt.query([]).unwrap();
                    let mut list = Vec::new();
                    while let Some(row) = rows.next().unwrap() {
                        let id: i64 = row.get(0).unwrap();
                        let name: String = row.get(1).unwrap();
                        let url: String = row.get(2).unwrap();
                        let event_types: String = row.get(3).unwrap();
                        let is_active: i32 = row.get(4).unwrap();

                        let subscribed = event_types.split(',').any(|e| e.trim() == event_type);
                        if subscribed {
                            list.push(Webhook { id, name, url, event_types, is_active });
                        }
                    }
                    list
                };

                let provider = if new_c.vmid < 0 { "Docker" } else { "Proxmox LXC" };
                for wh in webhooks {
                    let url = wh.url.clone();
                    let name = wh.name.clone();
                    let event = event_type.to_string();
                    let container_name = new_c.name.clone();
                    let vmid = new_c.vmid;
                    let status_str = new_status.clone();
                    let provider_str = provider.to_string();

                    tokio::spawn(async move {
                        send_webhook_notification(
                            url,
                            name,
                            &event,
                            &container_name,
                            vmid,
                            &status_str,
                            &provider_str,
                        )
                        .await;
                    });
                }
            }
        }
    }
}

fn handle_new_telemetry(state: &Arc<AppState>, metrics: AgentTelemetry) {
    let old_metrics = {
        let lock = state.latest_telemetry.read().unwrap();
        lock.clone()
    };
    if !old_metrics.lxc_containers.is_empty() {
        check_container_alerts(&old_metrics, &metrics, state);
    }
    *state.latest_telemetry.write().unwrap() = metrics;
}

fn handle_agent_connection_change(state: &Arc<AppState>, connected: bool) {
    let was_connected = {
        let mut conn_lock = state.agent_connected.write().unwrap();
        let old = *conn_lock;
        *conn_lock = connected;
        old
    };

    if was_connected != connected {
        let event_type = if connected { "agent_connected" } else { "agent_disconnected" };

        let webhooks = {
            let db = state.db.lock().unwrap();
            let mut stmt = db.prepare("SELECT id, name, url, event_types, is_active FROM webhooks WHERE is_active = 1").unwrap();
            let mut rows = stmt.query([]).unwrap();
            let mut list = Vec::new();
            while let Some(row) = rows.next().unwrap() {
                let id: i64 = row.get(0).unwrap();
                let name: String = row.get(1).unwrap();
                let url: String = row.get(2).unwrap();
                let event_types: String = row.get(3).unwrap();
                let is_active: i32 = row.get(4).unwrap();

                if event_types.split(',').any(|e| e.trim() == event_type) {
                    list.push(Webhook { id, name, url, event_types, is_active });
                }
            }
            list
        };

        let status_text = if connected { "online" } else { "offline" };

        for wh in webhooks {
            let url = wh.url.clone();
            let name = wh.name.clone();
            let event = event_type.to_string();
            let status_str = status_text.to_string();

            tokio::spawn(async move {
                send_webhook_notification(
                    url,
                    name,
                    &event,
                    "AMUD-Agent Daemon",
                    0,
                    &status_str,
                    "System",
                )
                .await;
            });
        }
    }
}

// Metrics collector listener task
fn start_agent_listener(state: Arc<AppState>) {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            let socket_path = std::env::var("AMUD_SOCKET_PATH")
                .unwrap_or_else(|_| "/opt/amud/run/amud.sock".to_string());
            run_uds_listener(
                &socket_path,
                state.clone(),
            )
            .await;
        }

        #[cfg(windows)]
        {
            let addr =
                std::env::var("AMUD_TCP_ADDR").unwrap_or_else(|_| "127.0.0.1:8050".to_string());
            run_tcp_listener(
                &addr,
                state.clone(),
            )
            .await;
        }
    });
}


#[cfg(unix)]
async fn run_uds_listener(
    path: &str,
    state: Arc<AppState>,
) {
    let uds_path = if FilePath::new(path)
        .parent()
        .map(|p| p.exists())
        .unwrap_or(false)
    {
        path
    } else {
        "/tmp/amud.sock"
    };

    println!(
        "Starting agent listener via UNIX Domain Socket at {}",
        uds_path
    );
    fs::remove_file(uds_path).ok();

    let listener = match TokioUnixListener::bind(uds_path) {
        Ok(l) => {
            fs::set_permissions(uds_path, std::fs::Permissions::from_mode(0o660)).ok();
            l
        }
        Err(e) => {
            eprintln!("UDS bind failed: {}. Telemetry offline listener active.", e);
            return;
        }
    };

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            println!("AMUD-Agent telemetry client UDS stream accepted.");
            handle_agent_connection_change(&state, true);

            let (reader, mut writer) = stream.into_split();
            let state_clone = state.clone();

            // Set up communication channel
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            *state.agent_command_tx.lock().unwrap() = Some(tx.clone());

            // Spawn writer task
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                while let Some(cmd) = rx.recv().await {
                    if let Err(e) = writer.write_all(cmd.as_bytes()).await {
                        eprintln!("Failed to write command to UDS: {}", e);
                        break;
                    }
                    if let Err(e) = writer.flush().await {
                        eprintln!("Failed to flush command to UDS: {}", e);
                        break;
                    }
                }
            });

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut reader = tokio::io::BufReader::new(reader);
                let mut line = String::new();
                let mut authenticated = state_clone.agent_secret.is_empty();

                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break; // EOF
                    }

                    if !authenticated {
                        if agent_authenticated(&state_clone.agent_secret, &line) {
                            authenticated = true;
                            line.clear();
                            continue;
                        }
                        println!("AMUD-Agent rejected: invalid IPC authentication.");
                        break;
                    }

                    process_agent_line(&state_clone, &tx, &line);
                    line.clear();
                }
                println!("AMUD-Agent telemetry client disconnected.");
                handle_agent_connection_change(&state_clone, false);
                *state_clone.agent_command_tx.lock().unwrap() = None; // clear command tx
            });
        }
    }
}

fn process_agent_line(
    state: &Arc<AppState>,
    tx: &tokio::sync::mpsc::UnboundedSender<String>,
    line: &str,
) {
    #[derive(Deserialize)]
    struct ConfigReq {
        request: String,
    }
    #[derive(Deserialize)]
    struct PveTestMsg {
        test_pve_result: PveTestResult,
    }

    if let Ok(req) = serde_json::from_str::<ConfigReq>(line) {
        if req.request == "get_config" {
            let token = {
                let db_lock = state.db.lock().unwrap();
                let mut stmt = db_lock
                    .prepare("SELECT value FROM settings WHERE key = 'pve_api_token'")
                    .unwrap();
                stmt.query_row([], |row| row.get::<_, String>(0))
                    .unwrap_or_default()
            };
            let config_payload = serde_json::json!({
                "config": {
                    "pve_api_token": token
                }
            });
            if let Ok(mut serialized) = serde_json::to_vec(&config_payload) {
                serialized.push(b'\n');
                let _ = tx.send(String::from_utf8_lossy(&serialized).into_owned());
            }
        }
    } else if let Ok(msg) = serde_json::from_str::<PveTestMsg>(line) {
        *state.pve_test_response.write().unwrap() = Some(msg.test_pve_result);
    } else if let Ok(metrics) = serde_json::from_str::<AgentTelemetry>(line) {
        handle_new_telemetry(state, metrics);
    }
}

// For fallback or cross-compiles
#[cfg(not(unix))]
#[allow(dead_code)]
async fn run_uds_listener(
    _path: &str,
    _state: Arc<AppState>,
) {
}

#[allow(dead_code)]
async fn run_tcp_listener(
    addr: &str,
    state: Arc<AppState>,
) {
    println!("Starting agent listener via TCP loopback on {}", addr);
    let listener = match TokioTcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("TCP loopback bind failed: {}.", e);
            return;
        }
    };

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            println!("AMUD-Agent telemetry client TCP stream accepted.");
            handle_agent_connection_change(&state, true);

            let (reader, mut writer) = stream.into_split();
            let state_clone = state.clone();

            // Set up communication channel
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            *state.agent_command_tx.lock().unwrap() = Some(tx.clone());

            // Spawn writer task
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                while let Some(cmd) = rx.recv().await {
                    if let Err(e) = writer.write_all(cmd.as_bytes()).await {
                        eprintln!("Failed to write command to TCP: {}", e);
                        break;
                    }
                    if let Err(e) = writer.flush().await {
                        eprintln!("Failed to flush command to TCP: {}", e);
                        break;
                    }
                }
            });

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut reader = tokio::io::BufReader::new(reader);
                let mut line = String::new();
                let mut authenticated = state_clone.agent_secret.is_empty();

                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break; // EOF
                    }

                    if !authenticated {
                        if agent_authenticated(&state_clone.agent_secret, &line) {
                            authenticated = true;
                            line.clear();
                            continue;
                        }
                        println!("AMUD-Agent rejected: invalid IPC authentication.");
                        break;
                    }

                    process_agent_line(&state_clone, &tx, &line);
                    line.clear();
                }
                println!("AMUD-Agent telemetry client disconnected.");
                handle_agent_connection_change(&state_clone, false);
                *state_clone.agent_command_tx.lock().unwrap() = None; // clear command tx
            });
        }
    }
}


fn get_overlay_gradient(theme: &str, custom_color: Option<&str>) -> String {
    match theme.to_lowercase().as_str() {
        "aurora" => "linear-gradient(135deg, rgba(4, 15, 15, 0.88) 0%, rgba(6, 24, 20, 0.82) 100%)"
            .to_string(),
        "crimson" => {
            "linear-gradient(135deg, rgba(18, 8, 8, 0.88) 0%, rgba(12, 10, 15, 0.82) 100%)"
                .to_string()
        }
        "obsidian" => {
            "linear-gradient(135deg, rgba(10, 10, 12, 0.92) 0%, rgba(15, 15, 18, 0.88) 100%)"
                .to_string()
        }
        "sunset" => "linear-gradient(135deg, rgba(20, 8, 12, 0.88) 0%, rgba(8, 10, 20, 0.82) 100%)"
            .to_string(),
        "custom" => {
            if let Some(hex) = custom_color {
                if hex.starts_with('#') && hex.len() == 7 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[1..3], 16),
                        u8::from_str_radix(&hex[3..5], 16),
                        u8::from_str_radix(&hex[5..7], 16),
                    ) {
                        return format!(
                            "linear-gradient(135deg, rgba({}, {}, {}, 0.88) 0%, rgba({}, {}, {}, 0.82) 100%)",
                            r / 2, g / 2, b / 2, r / 3, g / 3, b / 3
                        );
                    }
                }
            }
            "linear-gradient(135deg, rgba(8, 10, 18, 0.85) 0%, rgba(15, 10, 25, 0.8) 100%)"
                .to_string()
        }
        _ => "linear-gradient(135deg, rgba(8, 10, 18, 0.85) 0%, rgba(15, 10, 25, 0.8) 100%)"
            .to_string(),
    }
}

// Handlers
async fn dashboard_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);

    // Load Settings
    let mut settings = HashMap::new();
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT key, value FROM settings").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let key: String = row.get(0).unwrap();
            let value: String = row.get(1).unwrap();
            settings.insert(key, value);
        }
    }

    // Populate default placeholders if missing
    let app_name = settings
        .get("app_name")
        .map(|s| s.as_str())
        .unwrap_or("AMUD");
    let tagline = settings
        .get("tagline")
        .map(|s| s.as_str())
        .unwrap_or("Homelab Operations Cockpit");
    let mut custom_bg_url = settings
        .get("custom_bg_url")
        .map(|s| s.as_str())
        .unwrap_or("/static/wallpaper.png");
    if custom_bg_url.is_empty()
        || custom_bg_url
            == "https://raw.githubusercontent.com/youssef-boubli/assets/main/dashboard-bg.jpg"
    {
        custom_bg_url = "/static/wallpaper.png";
    }
    let app_logo = settings.get("app_logo").map(|s| s.as_str()).unwrap_or("");
    let accent_color = settings
        .get("accent_color")
        .map(|s| s.as_str())
        .unwrap_or("#cf6427");
    let glass_blur = settings
        .get("glass_blur_intensity")
        .map(|s| s.as_str())
        .unwrap_or("16");
    let glass_opacity = settings
        .get("glass_opacity")
        .map(|s| s.as_str())
        .unwrap_or("0.45");
    let bento_radius = settings
        .get("bento_radius")
        .map(|s| s.as_str())
        .unwrap_or("16");
    let grid_columns = settings
        .get("grid_columns")
        .or_else(|| settings.get("app_grid_columns"))
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|cols| (2..=5).contains(cols))
        .unwrap_or(3);

    let overlay_theme = settings
        .get("overlay_theme")
        .map(|s| s.as_str())
        .unwrap_or("cyber");
    let custom_overlay_color = settings
        .get("custom_overlay_color")
        .map(|s| s.as_str())
        .unwrap_or("#1a1a2e");
    let weather_lat = settings
        .get("weather_latitude")
        .map(|s| s.as_str())
        .unwrap_or("");
    let weather_lon = settings
        .get("weather_longitude")
        .map(|s| s.as_str())
        .unwrap_or("");
    let is_admin = session.as_ref().map(|s| s.role == "Admin").unwrap_or(false);
    let pve_api_token = settings
        .get("pve_api_token")
        .map(|s| s.as_str())
        .unwrap_or("");

    // Load Categories from DB for dropdown
    let mut db_categories = Vec::<(i64, String)>::new();
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db
            .prepare("SELECT id, name FROM categories ORDER BY sort_order ASC, name ASC")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let id: i64 = row.get(0).unwrap();
            let name: String = row.get(1).unwrap();
            db_categories.push((id, name));
        }
    }
    let mut category_options_html = String::new();
    for (_id, cat_name) in &db_categories {
        category_options_html.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            escape_html(cat_name),
            escape_html(cat_name)
        ));
    }
    if category_options_html.is_empty() {
        category_options_html = r#"<option value="General">General</option>"#.to_string();
    }

    // Load Applications
    let apps_html;
    let mut apps = Vec::<App>::new();
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT id, name, url, icon, description, category, node_tag FROM apps ORDER BY id DESC").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            apps.push(App {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                url: row.get(2).unwrap(),
                icon: row.get(3).unwrap(),
                description: row.get(4).unwrap(),
                category: row.get(5).unwrap(),
                node_tag: row.get(6).unwrap(),
            });
        }
    }

    if apps.is_empty() {
        apps_html = r#"
        <div class="glass-panel app-card" style="grid-column: span 3; text-align: center; padding: 3rem 1rem;">
            <p style="font-weight: 600; color: var(--text-secondary);">No services configured yet</p>
            <p style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.5rem;">Log in as Admin and click "Add App" to register your infrastructure.</p>
        </div>"#.to_string();
    } else {
        // Group cards into the configured number of dashboard columns.
        let mut cols = vec![String::new(); grid_columns];
        for (i, app) in apps.iter().enumerate() {
            let col_idx = i % grid_columns;

            // Resolve Built-in Brand Logo
            let lowercase_icon = app.icon.to_lowercase();
            let mut resolved_logo = String::new();
            if app.icon.starts_with("http") || app.icon.starts_with("/") {
                resolved_logo = app.icon.clone();
            } else if !lowercase_icon.is_empty() {
                let possible_paths = [
                    format!("ui/static/logos/{}.svg", lowercase_icon),
                    format!("ui/static/logos/{}.png", lowercase_icon),
                    format!("ui/static/logos/{}.jpg", lowercase_icon),
                    format!("ui/static/logos/{}.svg", lowercase_icon.replace(' ', "-")),
                    format!("ui/static/logos/{}.png", lowercase_icon.replace(' ', "-")),
                    format!("static/logos/{}.svg", lowercase_icon),
                    format!("static/logos/{}.png", lowercase_icon),
                ];
                for path in &possible_paths {
                    if std::path::Path::new(path).exists() {
                        let web_path = if path.starts_with("ui/") {
                            path["ui".len()..].to_string()
                        } else {
                            format!("/{}", path)
                        };
                        resolved_logo = web_path;
                        break;
                    }
                }
            }
            let brand_logo = if !resolved_logo.is_empty() {
                resolved_logo
            } else {
                match lowercase_icon.as_str() {
                    "plex" => "/static/logos/plex.svg".to_string(),
                    "jellyfin" => "/static/logos/jellyfin.svg".to_string(),
                    "proxmox" => "/static/logos/proxmox.svg".to_string(),
                    "portainer" => "/static/logos/portainer.svg".to_string(),
                    "home-assistant" | "homeassistant" => {
                        "/static/logos/home-assistant.svg".to_string()
                    }
                    "nextcloud" => "/static/logos/nextcloud.svg".to_string(),
                    "adguard" | "adguard-home" => "/static/logos/adguard-home.svg".to_string(),
                    "pihole" | "pi-hole" => "/static/logos/pi-hole.svg".to_string(),
                    "sonarr" => "/static/logos/sonarr.svg".to_string(),
                    "radarr" => "/static/logos/radarr.svg".to_string(),
                    "qbittorrent" => "/static/logos/qbittorrent.svg".to_string(),
                    "transmission" => "/static/logos/transmission.svg".to_string(),
                    "overseerr" => "/static/logos/overseerr.svg".to_string(),
                    "truenas" => "/static/logos/truenas.svg".to_string(),
                    "casaos" => "/static/logos/casaos.svg".to_string(),
                    _ => "/static/fallback.svg".to_string(),
                }
            };

            // Status indicator
            let status_badge = if app.name.to_lowercase().contains("proxmox") {
                r#"<span class="status-badge ms">173 ms</span>"#
            } else if app.name.to_lowercase().contains("truenas") {
                r#"<span class="status-badge ms">255 ms</span>"#
            } else if app.name.to_lowercase().contains("portainer") {
                r#"<span class="status-badge ms">452 ms</span>"#
            } else {
                r#"<span class="status-badge" style="background:rgba(255,255,255,0.05);color:var(--text-muted);border:1px solid var(--border-card);">CHECKING...</span>"#
            };

            // Category slug for filtering
            let cat_slug: String = app
                .category
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();

            // Build Sub-Metrics Grid
            let name_lower = app.name.to_lowercase();
            let sub_metrics = if name_lower.contains("proxmox") {
                r#"
                <div class="nested-metrics-grid cols-3">
                    <div class="metric-block">
                        <span class="metric-value">4 / 5</span>
                        <span class="metric-label">VMs</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">2%</span>
                        <span class="metric-label">CPU</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">96%</span>
                        <span class="metric-label">Mem</span>
                    </div>
                </div>"#
                    .to_string()
            } else {
                r#"
                <div class="nested-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value">Bookmark</span>
                        <span class="metric-label">Type</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">Linked</span>
                        <span class="metric-label">Status</span>
                    </div>
                </div>"#
                    .to_string()
            };

            let delete_btn = if is_admin {
                let app_json = serde_json::to_string(&app).unwrap_or_default();
                let escaped_json = app_json
                    .replace('&', "&amp;")
                    .replace('"', "&quot;")
                    .replace('\'', "&#39;");
                format!(
                    r#"
                    <div style="display: inline-flex; align-items: center; gap: 0.25rem;">
                        <button type="button" class="btn-edit-app" title="Edit application" data-app="{}" @click="editApp = JSON.parse($el.getAttribute('data-app')); editAppModalOpen = true;">
                            <i data-lucide="edit-2"></i>
                        </button>
                        <form action="/apps/delete" method="POST" style="margin: 0; display: inline-flex; align-items: center;">
                            <input type="hidden" name="id" value="{}">
                            <button type="submit" class="btn-delete-app" title="Delete application">
                                <i data-lucide="trash-2"></i>
                            </button>
                        </form>
                    </div>
                    "#,
                    escaped_json, app.id
                )
            } else {
                "".to_string()
            };
            let ctrl_container = if is_admin {
                r#"
                <div class="container-controls" style="display: none; align-items: center; gap: 0.25rem;" data-id="" data-provider="">
                    <button type="button" class="btn-ctrl start" title="Start Container" onclick="triggerContainerAction(this, 'start')">
                        <i data-lucide="circle-play" style="width:0.9rem; height:0.9rem;"></i>
                    </button>
                    <button type="button" class="btn-ctrl stop" title="Stop Container" onclick="triggerContainerAction(this, 'stop')">
                        <i data-lucide="circle-stop" style="width:0.9rem; height:0.9rem;"></i>
                    </button>
                    <button type="button" class="btn-ctrl restart" title="Restart Container" onclick="triggerContainerAction(this, 'restart')">
                        <i data-lucide="rotate-cw" style="width:0.9rem; height:0.9rem;"></i>
                    </button>
                </div>
                "#.to_string()
            } else {
                "".to_string()
            };

            let card = format!(
                r#"
                <div class="glass-panel app-card" data-app-name="{}" data-category="{}">
                    <div class="app-card-header">
                        <a href="{}" target="_blank" rel="noopener noreferrer" class="app-card-identity" style="text-decoration:none; color:inherit;">
                            <div class="app-card-icon">
                                <img src="{}" onerror="this.src='/static/fallback.svg'">
                            </div>
                            <div>
                                <h3 class="app-card-title">{}</h3>
                                <p class="app-card-desc">{}</p>
                            </div>
                        </a>
                        <div style="display: flex; align-items: center; gap: 0.5rem;" class="app-card-badges">
                            {}
                            {}
                            {}
                        </div>
                    </div>
                    {}
                </div>"#,
                escape_html(&name_lower),
                escape_html(&cat_slug),
                escape_html(&app.url),
                escape_html(&brand_logo),
                escape_html(&app.name),
                escape_html(&app.description),
                status_badge,
                ctrl_container,
                delete_btn,
                sub_metrics
            );
            cols[col_idx].push_str(&card);
        }

        apps_html = cols
            .into_iter()
            .map(|col| format!(r#"<div class="bento-column">{}</div>"#, col))
            .collect::<Vec<_>>()
            .join("");
    }

    // Auth actions buttons in topbar
    let auth_buttons = if let Some(ref sess) = session {
        let admin_settings_btn = if sess.role == "Admin" {
            r#"
            <button type="button" class="glass-panel btn-admin" @click="addAppModalOpen = true" style="padding:0.5rem 1rem; border-radius:8px; background:rgba(255,255,255,0.02); font-weight:600; cursor:pointer; font-size:0.82rem; display:inline-flex; align-items:center; gap:0.35rem; color:#fff; border:1px solid rgba(255,255,255,0.06);">
                <i data-lucide="plus" style="width:0.95rem; height:0.95rem;"></i> Add App
            </button>
            <button type="button" class="glass-panel btn-admin" onclick="window.location.href='/admin/settings'" style="padding:0.5rem 1rem; border-radius:8px; background:rgba(255,255,255,0.02); font-weight:600; cursor:pointer; font-size:0.82rem; display:inline-flex; align-items:center; gap:0.35rem; color:#fff; border:1px solid rgba(255,255,255,0.06);">
                <i data-lucide="sliders-horizontal" style="width:0.95rem; height:0.95rem;"></i> Settings
            </button>
            "#
        } else {
            ""
        };
        format!(
            r#"
            {}
            <a href="/logout" class="glass-panel" style="padding:0.5rem 1rem; border-radius:8px; font-weight:600; font-size:0.82rem; text-decoration:none; color:var(--text-secondary); border:1px solid rgba(255,255,255,0.06); display:inline-flex; align-items:center; gap:0.35rem; background:rgba(255,255,255,0.02);">
                <i data-lucide="log-out" style="width:0.95rem; height:0.95rem;"></i> Sign Out ({})
            </a>
            "#,
            admin_settings_btn, sess.username
        )
    } else {
        r#"
        <a href="/login" class="glass-panel" style="padding:0.5rem 1rem; border-radius:8px; font-weight:600; font-size:0.82rem; text-decoration:none; color:#fff; border:1px solid rgba(255,255,255,0.06); display:inline-flex; align-items:center; gap:0.35rem; background:var(--accent-glow);">
            <i data-lucide="key-round" style="width:0.95rem; height:0.95rem;"></i> Sign In
        </a>
        "#.to_string()
    };

    // Scan Plex / Jellyfin apps presence
    let has_plex = apps
        .iter()
        .any(|app| app.name.to_lowercase().contains("plex"));
    let has_jellyfin = apps.iter().any(|app| {
        app.name.to_lowercase().contains("jellyfin") || app.name.to_lowercase().contains("emby")
    });

    let mut streams_html = String::new();
    if has_plex || has_jellyfin {
        let mut cards = String::new();
        if has_plex {
            cards.push_str(r#"
            <!-- Plex stream card -->
            <div class="glass-panel stream-card">
                <div class="stream-main">
                    <div class="stream-meta">
                        <div class="stream-icon">
                            <i data-lucide="play" style="color: #ff9900;"></i>
                        </div>
                        <div>
                            <h2 class="stream-text-title">Plex</h2>
                            <p class="stream-text-desc">Watch movies and TV shows.</p>
                        </div>
                    </div>
                    <span class="stream-status-badge" data-stream-app="plex" data-stream-service="plex">CHECKING...</span>
                </div>
                
                <div class="stream-player">
                    <div class="stream-controls-row">
                        <span class="stream-track-title" id="plex-track">No Active Streams</span>
                        <div style="display: flex; gap: 0.5rem; align-items: center;">
                            <button class="stream-play-btn"><i data-lucide="play" style="width:0.85rem; height:0.85rem;"></i></button>
                            <span id="plex-timer">-</span>
                        </div>
                    </div>
                    <div class="stream-progress-track">
                        <div class="stream-progress-fill" id="plex-progress" style="width: 0%;"></div>
                    </div>
                </div>
            </div>
            "#);
        }
        if has_jellyfin {
            cards.push_str(r#"
            <!-- Jellyfin/Emby stream card -->
            <div class="glass-panel stream-card">
                <div class="stream-main">
                    <div class="stream-meta">
                        <div class="stream-icon">
                            <i data-lucide="play-circle" style="color: #10b981;"></i>
                        </div>
                        <div>
                            <h2 class="stream-text-title">Jellyfin</h2>
                            <p class="stream-text-desc">Watch movies and TV shows.</p>
                        </div>
                    </div>
                    <span class="stream-status-badge" data-stream-app="jellyfin emby media" data-stream-service="emby">CHECKING...</span>
                </div>
                
                <div class="stream-player">
                    <div class="stream-controls-row">
                        <span class="stream-track-title" id="emby-track" style="color: var(--text-muted);">No Active Streams</span>
                        <span id="emby-timer">-</span>
                    </div>
                    <div class="stream-progress-track">
                        <div class="stream-progress-fill" id="emby-progress" style="width: 0%;"></div>
                    </div>
                </div>
            </div>
            "#);
        }

        let cols_class = if has_plex && has_jellyfin {
            "streams-row"
        } else {
            "streams-row single-col"
        };
        streams_html = format!(r#"<section class="{}">{}</section>"#, cols_class, cards);
    }

    // Build category filter tabs HTML
    let mut categories = Vec::<String>::new();
    for app in apps.iter() {
        if !app.category.is_empty() && !categories.contains(&app.category) {
            categories.push(app.category.clone());
        }
    }

    let mut category_tabs_html = format!(
        r#"<button class="filter-tab active" onclick="filterCategory('all', this)">All <span class="filter-count">{}</span></button>"#,
        apps.len()
    );
    for cat in categories.iter() {
        let count = apps.iter().filter(|a| &a.category == cat).count();
        let cat_slug: String = cat
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        category_tabs_html.push_str(&format!(
            r#"<button class="filter-tab" onclick="filterCategory('{}', this)">{} <span class="filter-count">{}</span></button>"#,
            escape_html(&cat_slug), escape_html(cat), count
        ));
    }

    // Build Support / Donation card. The links are hardcoded to the AMUD author -
    // self-hosters can only enable or disable the card, not change the links.
    let donate_enabled = settings.get("donate_enabled").map(|s| s.as_str()).unwrap_or("1");
    let mut support_html = String::new();
    if donate_enabled == "1" {
        let mut links = String::new();
        for (url, label, icon) in DONATION_LINKS.iter() {
            links.push_str(&format!(
                r#"<a href="{}" target="_blank" rel="noopener noreferrer" class="support-link"><i data-lucide="{}" style="width:1rem; height:1rem;"></i> {}</a>"#,
                url, icon, label
            ));
        }
        support_html = format!(
            r#"<section class="support-section">
                <div class="glass-panel support-card">
                    <div class="support-head">
                        <i data-lucide="heart" style="color:var(--accent-color); width:1.2rem; height:1.2rem;"></i>
                        <h2>Support AMUD</h2>
                    </div>
                    <p class="support-msg">{}</p>
                    <div class="support-links">{}</div>
                </div>
            </section>"#,
            DONATION_MESSAGE, links
        );
    }

    // Build root_css style overrides
    let bg_url_style = if custom_bg_url.is_empty() {
        "".to_string()
    } else {
        format!("--brand-bg-image: url('{}');", custom_bg_url)
    };
    let logo_url_style = if app_logo.is_empty() {
        "".to_string()
    } else {
        format!("--brand-logo-url: url('{}');", app_logo)
    };

    let opacity_f: f64 = glass_opacity.parse().unwrap_or(0.45);
    let accent_glow = if accent_color.starts_with('#') && accent_color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&accent_color[1..3], 16),
            u8::from_str_radix(&accent_color[3..5], 16),
            u8::from_str_radix(&accent_color[5..7], 16),
        ) {
            format!("rgba({}, {}, {}, 0.15)", r, g, b)
        } else {
            "rgba(56, 189, 248, 0.15)".to_string()
        }
    } else {
        "rgba(56, 189, 248, 0.15)".to_string()
    };

    let overlay_gradient = get_overlay_gradient(overlay_theme, Some(custom_overlay_color));

    let root_css = format!(
        r#"
            {}
            {}
            --brand-title: "{}";
            --brand-slogan: "{}";
            --accent-color: {};
            --accent-glow: {};
            --glass-blur-intensity: {}px;
            --glass-opacity: {};
            --radius-xl: {}px;
            --grid-cols: {};
            --bg-card: rgba(15, 20, 25, {});
            --brand-overlay-gradient: {};
        "#,
        bg_url_style,
        logo_url_style,
        app_name,
        tagline,
        accent_color,
        accent_glow,
        glass_blur,
        glass_opacity,
        bento_radius,
        grid_columns,
        opacity_f,
        overlay_gradient
    );

    // Load templates
    let index_tmpl = include_str!("../../ui/templates/index.html");
    let username = session
        .as_ref()
        .map(|s| s.username.as_str())
        .unwrap_or("guest");
    let result = index_tmpl
        .replace("/* ROOT_CSS */", &root_css)
        .replace("{{app_name}}", app_name)
        .replace("{{tagline}}", tagline)
        .replace("{{custom_bg_url}}", custom_bg_url)
        .replace("{{app_logo}}", app_logo)
        .replace("{{if app_logo}}", if app_logo.is_empty() { "" } else { "" })
        .replace("{{end}}", "")
        .replace("{{accent_color}}", accent_color)
        .replace("{{glass_blur_intensity}}", glass_blur)
        .replace("{{glass_opacity}}", glass_opacity)
        .replace("{{bento_radius}}", bento_radius)
        .replace("<!-- APPS_GRID -->", &apps_html)
        .replace("<!-- STREAMS_ROW -->", &streams_html)
        .replace("<!-- CATEGORY_TABS -->", &category_tabs_html)
        .replace("<!-- SUPPORT_SECTION -->", &support_html)
        .replace("<!-- AUTH_BUTTONS -->", &auth_buttons)
        .replace("{{username}}", username)
        .replace(
            "{{eq_cyber}}",
            if overlay_theme == "cyber" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_aurora}}",
            if overlay_theme == "aurora" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_crimson}}",
            if overlay_theme == "crimson" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_sunset}}",
            if overlay_theme == "sunset" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_obsidian}}",
            if overlay_theme == "obsidian" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_custom}}",
            if overlay_theme == "custom" {
                "selected"
            } else {
                ""
            },
        )
        .replace("{{custom_overlay_color}}", custom_overlay_color)
        .replace("{{weather_latitude}}", weather_lat)
        .replace("{{weather_longitude}}", weather_lon)
        .replace("<!-- CATEGORY_OPTIONS -->", &category_options_html)
        .replace("{{pve_api_token}}", pve_api_token)
        .replace("{{is_admin}}", if is_admin { "true" } else { "false" });

    Html(result)
}

async fn login_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Load Settings
    let mut settings = HashMap::new();
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT key, value FROM settings").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let key: String = row.get(0).unwrap();
            let value: String = row.get(1).unwrap();
            settings.insert(key, value);
        }
    }

    let mut custom_bg_url = settings
        .get("custom_bg_url")
        .map(|s| s.as_str())
        .unwrap_or("/static/wallpaper.png");
    if custom_bg_url.is_empty()
        || custom_bg_url
            == "https://raw.githubusercontent.com/youssef-boubli/assets/main/dashboard-bg.jpg"
    {
        custom_bg_url = "/static/wallpaper.png";
    }
    let app_logo = settings.get("app_logo").map(|s| s.as_str()).unwrap_or("");
    let app_name = settings
        .get("app_name")
        .map(|s| s.as_str())
        .unwrap_or("AMUD");
    let accent_color = settings
        .get("accent_color")
        .map(|s| s.as_str())
        .unwrap_or("#cf6427");
    let glass_blur = settings
        .get("glass_blur_intensity")
        .map(|s| s.as_str())
        .unwrap_or("16");
    let glass_opacity = settings
        .get("glass_opacity")
        .map(|s| s.as_str())
        .unwrap_or("0.45");
    let bento_radius = settings
        .get("bento_radius")
        .map(|s| s.as_str())
        .unwrap_or("16");
    let overlay_theme = settings
        .get("overlay_theme")
        .map(|s| s.as_str())
        .unwrap_or("cyber");

    let bg_url_style = if custom_bg_url.is_empty() {
        "".to_string()
    } else {
        format!("--brand-bg-image: url('{}');", custom_bg_url)
    };
    let logo_url_style = if app_logo.is_empty() {
        "".to_string()
    } else {
        format!("--brand-logo-url: url('{}');", app_logo)
    };

    let opacity_f: f64 = glass_opacity.parse().unwrap_or(0.45);
    let accent_glow = if accent_color.starts_with('#') && accent_color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&accent_color[1..3], 16),
            u8::from_str_radix(&accent_color[3..5], 16),
            u8::from_str_radix(&accent_color[5..7], 16),
        ) {
            format!("rgba({}, {}, {}, 0.15)", r, g, b)
        } else {
            "rgba(56, 189, 248, 0.15)".to_string()
        }
    } else {
        "rgba(56, 189, 248, 0.15)".to_string()
    };

    let custom_overlay_color = settings
        .get("custom_overlay_color")
        .map(|s| s.as_str())
        .unwrap_or("#1a1a2e");
    let overlay_gradient = get_overlay_gradient(overlay_theme, Some(custom_overlay_color));

    let root_css = format!(
        r#"
            {}
            {}
            --brand-title: "{}";
            --accent-color: {};
            --accent-glow: {};
            --glass-blur-intensity: {}px;
            --glass-opacity: {};
            --radius-xl: {}px;
            --bg-card: rgba(15, 20, 25, {});
            --brand-overlay-gradient: {};
        "#,
        bg_url_style,
        logo_url_style,
        app_name,
        accent_color,
        accent_glow,
        glass_blur,
        glass_opacity,
        bento_radius,
        opacity_f,
        overlay_gradient
    );

    let login_tmpl = include_str!("../../ui/templates/login.html");
    let result = login_tmpl.replace("/* ROOT_CSS */", &root_css);
    Html(result)
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let username = form.get("username").cloned().unwrap_or_default().trim().to_string();
    let password = form.get("password").cloned().unwrap_or_default();

    if login_rate_limited(&state.login_attempts, &username) {
        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(axum::body::Body::from("Too many failed login attempts. Try again later."))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT password_hash, role FROM users WHERE username = ?")
        .unwrap();

    let auth_res = stmt.query_row(params![username.clone()], |row| {
        let pwhash: String = row.get(0).unwrap();
        let role: String = row.get(1).unwrap();
        let (verified, needs_rehash) = verify_password(&pwhash, &password);
        Ok((verified, needs_rehash, role))
    });

    if let Ok((true, needs_rehash, role)) = auth_res {
        if needs_rehash {
            let upgraded = hash_password(&password);
            db.execute(
                "UPDATE users SET password_hash = ? WHERE username = ?",
                params![upgraded, username],
            )
            .ok();
        }
        clear_failed_logins(&state.login_attempts, &username);
        let token = generate_session_token();

        state
            .sessions
            .write()
            .unwrap()
            .insert(
                token.clone(),
                Session {
                    username,
                    role,
                    expires_at_epoch: now_epoch_secs() + 86_400,
                },
            );

        let cookie = session_cookie(&token);

        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::SET_COOKIE, cookie)
            .header(header::LOCATION, "/")
            .body(axum::body::Body::empty())
            .unwrap()
    } else {
        // Keep missing-user and wrong-password timing closer by doing an Argon2id hash
        // even when no stored hash exists.
        let _ = hash_password(&password);
        record_failed_login(&state.login_attempts, &username);
        Redirect::to("/login").into_response()
    }
}

async fn logout_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(cookie_header) = headers.get("cookie").and_then(|c| c.to_str().ok()) {
        if let Some(token) = cookie_header
            .split(';')
            .map(|s| s.trim())
            .find(|s| s.starts_with("amud_session="))
            .map(|s| s["amud_session=".len()..].to_string())
        {
            state.sessions.write().unwrap().remove(&token);
        }
    }

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::SET_COOKIE, expired_session_cookie())
        .header(header::LOCATION, "/")
        .body(axum::body::Body::empty())
        .unwrap()
}

// WS upgrades handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws_session(socket, state))
}

async fn handle_ws_session(mut socket: WebSocket, state: Arc<AppState>) {
    let rx_stream = state.latest_telemetry.clone();

    loop {
        // Stream telemetry packet every 3 seconds
        let system_metrics = rx_stream.read().unwrap().clone();
        let network = system_metrics.network.clone().unwrap_or_default();
        let streams = state.media_streams.read().unwrap().clone();
        let app_statuses = state.app_statuses.read().unwrap().clone();

        let payload = FullTelemetry {
            system: system_metrics,
            network,
            streams,
            app_statuses,
        };

        if let Ok(msg) = serde_json::to_string(&payload) {
            if socket.send(WsMessage::Text(msg)).await.is_err() {
                break;
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

// Settings Handler
async fn settings_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if session.map(|s| s.role == "Admin").unwrap_or(false) {
        let db = state.db.lock().unwrap();
        let mut new_token = None;
        for (key, val) in form {
            // Skip any password fields â€” credentials are changed via /admin/credentials
            if key == "new_password"
                || key == "old_password"
                || key == "repeat_password"
                || key == "new_username"
            {
                continue;
            }
            if key == "pve_api_token" {
                new_token = Some(val.clone());
            }
            db.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = ?2",
                params![key, val],
            )
            .ok();
        }

        if let Some(token) = new_token {
            let config_payload = serde_json::json!({
                "config": {
                    "pve_api_token": token
                }
            });
            if let Ok(mut serialized) = serde_json::to_vec(&config_payload) {
                serialized.push(b'\n');
                if let Some(tx) = &*state.agent_command_tx.lock().unwrap() {
                    let _ = tx.send(String::from_utf8_lossy(&serialized).into_owned());
                }
            }
        }
    }
    Redirect::to("/admin/settings")
}

// Proxmox VE API Token connection tester handler
async fn test_proxmox_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let token = form.get("pve_api_token").cloned().unwrap_or_default();

    // Clear any previous test response
    *state.pve_test_response.write().unwrap() = None;

    // Send test_pve command to the agent
    let cmd = serde_json::json!({
        "action": "test_pve",
        "id": token
    });

    let mut success = false;
    let mut error = None;

    if let Ok(mut serialized) = serde_json::to_vec(&cmd) {
        serialized.push(b'\n');

        let sent = {
            if let Some(tx) = &*state.agent_command_tx.lock().unwrap() {
                tx.send(String::from_utf8_lossy(&serialized).into_owned())
                    .is_ok()
            } else {
                false
            }
        };

        if sent {
            // Wait for response up to 5 seconds
            let start = std::time::Instant::now();
            while start.elapsed() < std::time::Duration::from_secs(5) {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(res) = state.pve_test_response.read().unwrap().as_ref() {
                    success = res.success;
                    error = res.error.clone();
                    break;
                }
            }
            if !success && error.is_none() {
                error = Some("Connection test timed out waiting for agent response".to_string());
            }
        } else {
            error = Some("AMUD Agent is offline or not connected".to_string());
        }
    } else {
        error = Some("Failed to serialize test command".to_string());
    }

    let body = serde_json::json!({
        "success": success,
        "error": error
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&body).unwrap(),
        ))
        .unwrap()
}

// Secure Credentials Update Handler
async fn credentials_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    let sess = match session {
        Some(ref s) if s.role == "Admin" => s,
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };

    let old_password = form.get("old_password").cloned().unwrap_or_default();
    let new_password = form.get("new_password").cloned().unwrap_or_default();
    let new_username = form.get("new_username").cloned().unwrap_or_default();

    if old_password.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Old password is required"}"#,
            ))
            .unwrap();
    }

    let db = state.db.lock().unwrap();

    // Verify old password matches the current user's password
    let stored_hash: Result<String, _> = db
        .prepare("SELECT password_hash FROM users WHERE username = ?")
        .unwrap()
        .query_row(params![sess.username], |row| row.get(0));

    let old_needs_rehash = match stored_hash {
        Ok(ref h) => {
            let (verified, needs_rehash) = verify_password(h, &old_password);
            if verified {
                needs_rehash
            } else {
                return Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"error":"Old password is incorrect"}"#,
                    ))
                    .unwrap();
            }
        }
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"Old password is incorrect"}"#,
                ))
                .unwrap();
        }
    };

    // Update username if provided (check uniqueness first)
    let mut actual_username = sess.username.clone();
    if !new_username.is_empty() && new_username != sess.username {
        let count: i64 = db
            .prepare("SELECT COUNT(*) FROM users WHERE username = ?")
            .unwrap()
            .query_row(params![new_username], |row| row.get(0))
            .unwrap_or(0);

        if count > 0 {
            return Response::builder()
                .status(StatusCode::CONFLICT)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"Username is already taken"}"#,
                ))
                .unwrap();
        }

        if db
            .execute(
                "UPDATE users SET username = ? WHERE username = ?",
                params![new_username, sess.username],
            )
            .is_ok()
        {
            actual_username = new_username.clone();

            // Also update the session in memory so that it matches the new username
            if let Some(cookie_header) = headers.get("cookie").and_then(|c| c.to_str().ok()) {
                if let Some(token) = cookie_header
                    .split(';')
                    .map(|s| s.trim())
                    .find(|s| s.starts_with("amud_session="))
                    .map(|s| s["amud_session=".len()..].to_string())
                {
                    if let Some(s) = state.sessions.write().unwrap().get_mut(&token) {
                        s.username = new_username.clone();
                    }
                }
            }
        }
    }

    // Update password if provided
    if !new_password.is_empty() || old_needs_rehash {
        let new_hash = hash_password(if new_password.is_empty() {
            &old_password
        } else {
            &new_password
        });
        db.execute(
            "UPDATE users SET password_hash = ? WHERE username = ?",
            params![new_hash, actual_username],
        )
        .ok();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

// Categories CRUD Handlers
async fn list_categories_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT id, name, color, sort_order FROM categories ORDER BY sort_order ASC, name ASC",
        )
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut categories = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id: i64 = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        let color: String = row.get(2).unwrap();
        let sort_order: i64 = row.get(3).unwrap();
        categories.push(serde_json::json!({
            "id": id,
            "name": name,
            "color": color,
            "sort_order": sort_order
        }));
    }
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&categories).unwrap(),
        ))
        .unwrap()
}

async fn add_category_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let name = form.get("name").cloned().unwrap_or_default();
    let color = form
        .get("color")
        .cloned()
        .unwrap_or_else(|| "#64748b".to_string());

    if name.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Category name is required"}"#,
            ))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT INTO categories (name, color) VALUES (?, ?)",
        params![name, color],
    ) {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::CONFLICT)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Category already exists"}"#,
            ))
            .unwrap(),
    }
}

async fn delete_category_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let db = state.db.lock().unwrap();
            db.execute("DELETE FROM categories WHERE id = ?", params![id])
                .ok();
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

async fn edit_category_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let name = form.get("name").cloned().unwrap_or_default();
            let color = form
                .get("color")
                .cloned()
                .unwrap_or_else(|| "#64748b".to_string());
            if !name.is_empty() {
                let db = state.db.lock().unwrap();
                db.execute(
                    "UPDATE categories SET name = ?, color = ? WHERE id = ?",
                    params![name, color, id],
                )
                .ok();
            }
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

// Add App Handler
async fn add_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if session.map(|s| s.role == "Admin").unwrap_or(false) {
        let name = form.get("name").cloned().unwrap_or_default();
        let url = normalize_url(&form.get("url").cloned().unwrap_or_default());
        let icon = form.get("icon").cloned().unwrap_or_default();
        let category = form
            .get("category")
            .cloned()
            .unwrap_or_else(|| "General".to_string());
        let node_tag = form
            .get("node_tag")
            .cloned()
            .unwrap_or_else(|| "Local".to_string());
        let description = form.get("description").cloned().unwrap_or_default();

        if !name.is_empty() && !url.is_empty() {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO apps (name, url, icon, description, category, node_tag) VALUES (?, ?, ?, ?, ?, ?)",
                params![name, url, icon, description, category, node_tag],
            )
            .ok();
        }
    }
    Redirect::to("/")
}

// Delete App Handler
async fn delete_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if session.map(|s| s.role == "Admin").unwrap_or(false) {
        if let Some(id_str) = form.get("id") {
            if let Ok(id) = id_str.parse::<i64>() {
                let db = state.db.lock().unwrap();
                db.execute("DELETE FROM apps WHERE id = ?", params![id])
                    .ok();
            }
        }
    }
    Redirect::to("/")
}

// Edit App Handler
async fn edit_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if session.map(|s| s.role == "Admin").unwrap_or(false) {
        if let Some(id_str) = form.get("id") {
            if let Ok(id) = id_str.parse::<i64>() {
                let name = form.get("name").cloned().unwrap_or_default();
                let url = normalize_url(&form.get("url").cloned().unwrap_or_default());
                let icon = form.get("icon").cloned().unwrap_or_default();
                let category = form
                    .get("category")
                    .cloned()
                    .unwrap_or_else(|| "General".to_string());
                let node_tag = form
                    .get("node_tag")
                    .cloned()
                    .unwrap_or_else(|| "Local".to_string());
                let description = form.get("description").cloned().unwrap_or_default();

                if !name.is_empty() && !url.is_empty() {
                    let db = state.db.lock().unwrap();
                    db.execute(
                        "UPDATE apps SET name = ?, url = ?, icon = ?, description = ?, category = ?, node_tag = ? WHERE id = ?",
                        params![name, url, icon, description, category, node_tag, id],
                    )
                    .ok();
                }
            }
        }
    }
    Redirect::to("/")
}

// Multipart File Uploader
async fn upload_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(axum::body::Body::from("Forbidden"))
            .unwrap();
    }

    let mut url_path = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            let filename_orig = field.file_name().unwrap_or("image.png").to_string();
            let ext = FilePath::new(&filename_orig)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
                .to_lowercase();

            if ext != "png"
                && ext != "jpg"
                && ext != "jpeg"
                && ext != "svg"
                && ext != "ico"
                && ext != "gif"
            {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from("Invalid file extension"))
                    .unwrap();
            }

            let bytes = match field.bytes().await {
                Ok(b) => b,
                Err(_) => {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(axum::body::Body::from("Failed reading image bytes"))
                        .unwrap();
                }
            };

            if bytes.len() > 5 * 1024 * 1024 {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from("File size exceeds 5MB limit"))
                    .unwrap();
            }

            fs::create_dir_all("data/uploads").ok();
            let nano = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let filename = format!("{}.{}", nano, ext);
            let filepath = format!("data/uploads/{}", filename);

            if fs::write(&filepath, bytes).is_ok() {
                url_path = format!("/uploads/{}", filename);
            }
        }
    }

    if url_path.is_empty() {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(axum::body::Body::from("No image uploaded"))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(format!(
                r#"{{"url":"{}"}}"#,
                url_path
            )))
            .unwrap()
    }
}

// Action Trigger Handler for LXC / Docker containers
async fn app_action_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if session.is_none() || session.unwrap().role != "Admin" {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Unauthorized"}"#))
            .unwrap();
    }

    let provider = form.get("provider").cloned().unwrap_or_default();
    let id = form.get("id").cloned().unwrap_or_default();
    let action = form.get("action").cloned().unwrap_or_default();

    if provider.is_empty() || id.is_empty() || action.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Missing parameters"}"#))
            .unwrap();
    }

    let cmd = format!(
        "{{\"provider\":\"{}\",\"id\":\"{}\",\"action\":\"{}\"}}\n",
        provider, id, action
    );

    let tx_guard = state.agent_command_tx.lock().unwrap();
    if let Some(ref tx) = *tx_guard {
        if tx.send(cmd).is_ok() {
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"success":true}"#))
                .unwrap();
        }
    }

    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"error":"Agent not connected"}"#))
        .unwrap()
}

// Webhook API handlers
async fn list_webhooks_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT id, name, url, event_types, is_active FROM webhooks ORDER BY id DESC")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut list = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id: i64 = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        let url: String = row.get(2).unwrap();
        let event_types: String = row.get(3).unwrap();
        let is_active: i32 = row.get(4).unwrap();

        let masked_url = if url.len() > 30 {
            let parsed = reqwest::Url::parse(&url);
            let host = parsed.as_ref().map(|u| u.host_str().unwrap_or("")).unwrap_or("");
            format!("{}://{}/...{}", if url.starts_with("https") { "https" } else { "http" }, host, &url[url.len().saturating_sub(8)..])
        } else {
            url.clone()
        };

        list.push(serde_json::json!({
            "id": id,
            "name": name,
            "url": url,
            "masked_url": masked_url,
            "event_types": event_types,
            "is_active": is_active
        }));
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&list).unwrap(),
        ))
        .unwrap()
}

async fn add_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let name = form.get("name").cloned().unwrap_or_default().trim().to_string();
    let url = form.get("url").cloned().unwrap_or_default().trim().to_string();
    let event_types = form.get("event_types").cloned().unwrap_or_default().trim().to_string();
    let is_active = form.get("is_active").and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);

    if name.is_empty() || url.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Name and URL are required"}"#))
            .unwrap();
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"URL must start with http:// or https://"}"#))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT INTO webhooks (name, url, event_types, is_active) VALUES (?, ?, ?, ?)",
        params![name, url, event_types, is_active],
    ) {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(format!(r#"{{"error":"Database error: {}"}}"#, e)))
            .unwrap(),
    }
}

async fn edit_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    let id = match id_str.parse::<i64>() {
        Ok(v) => v,
        Err(_) => return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Invalid Webhook ID"}"#))
            .unwrap()
    };

    let name = form.get("name").cloned().unwrap_or_default().trim().to_string();
    let url = form.get("url").cloned().unwrap_or_default().trim().to_string();
    let event_types = form.get("event_types").cloned().unwrap_or_default().trim().to_string();
    let is_active = form.get("is_active").and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);

    if name.is_empty() || url.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Name and URL are required"}"#))
            .unwrap();
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"URL must start with http:// or https://"}"#))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    match db.execute(
        "UPDATE webhooks SET name = ?, url = ?, event_types = ?, is_active = ? WHERE id = ?",
        params![name, url, event_types, is_active, id],
    ) {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(format!(r#"{{"error":"Database error: {}"}}"#, e)))
            .unwrap(),
    }
}

async fn delete_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    if let Ok(id) = id_str.parse::<i64>() {
        let db = state.db.lock().unwrap();
        db.execute("DELETE FROM webhooks WHERE id = ?", params![id]).ok();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

async fn test_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    let id = match id_str.parse::<i64>() {
        Ok(v) => v,
        Err(_) => return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Invalid ID"}"#))
            .unwrap()
    };

    let webhook = {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT name, url FROM webhooks WHERE id = ?").unwrap();
        stmt.query_row(params![id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).ok()
    };

    if let Some((name, url)) = webhook {
        tokio::spawn(async move {
            send_webhook_notification(
                url,
                name,
                "test",
                "Test Container",
                999,
                "running",
                "Docker",
            )
            .await;
        });

        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Webhook not found"}"#))
            .unwrap()
    }
}

// User Management Handlers
async fn list_users_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut users = Vec::new();
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT id, username, role FROM users").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let id: i64 = row.get(0).unwrap();
            let username: String = row.get(1).unwrap();
            let role: String = row.get(2).unwrap();
            users.push(serde_json::json!({ "id": id, "username": username, "role": role }));
        }
    }
    axum::response::Json(users)
}

#[derive(Deserialize)]
struct AddUserForm {
    username: String,
    password: Option<String>,
    role: String,
}

async fn add_user_handler(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AddUserForm>,
) -> impl IntoResponse {
    let pass = form.password.unwrap_or_default();
    if pass.is_empty() {
        return (StatusCode::BAD_REQUEST, "Password is required for new users.".to_string());
    }
    let p_hash = hash_password(&pass);
    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
        params![form.username.trim(), p_hash, form.role],
    ) {
        Ok(_) => (StatusCode::OK, "User added".to_string()),
        Err(_) => (StatusCode::BAD_REQUEST, "Username already exists or invalid.".to_string()),
    }
}

#[derive(Deserialize)]
struct EditUserForm {
    id: i64,
    username: String,
    password: Option<String>,
    role: String,
}

async fn edit_user_handler(
    State(state): State<Arc<AppState>>,
    Form(form): Form<EditUserForm>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    if let Some(pass) = form.password.filter(|p| !p.trim().is_empty()) {
        let p_hash = hash_password(&pass);
        match db.execute(
            "UPDATE users SET username = ?, password_hash = ?, role = ? WHERE id = ?",
            params![form.username.trim(), p_hash, form.role, form.id],
        ) {
            Ok(_) => (StatusCode::OK, "User updated".to_string()),
            Err(_) => (StatusCode::BAD_REQUEST, "Update failed.".to_string()),
        }
    } else {
        match db.execute(
            "UPDATE users SET username = ?, role = ? WHERE id = ?",
            params![form.username.trim(), form.role, form.id],
        ) {
            Ok(_) => (StatusCode::OK, "User updated".to_string()),
            Err(_) => (StatusCode::BAD_REQUEST, "Update failed.".to_string()),
        }
    }
}

#[derive(Deserialize)]
struct DeleteUserForm {
    id: i64,
}

async fn delete_user_handler(
    State(state): State<Arc<AppState>>,
    Form(form): Form<DeleteUserForm>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    db.execute("DELETE FROM users WHERE id = ?", params![form.id]).ok();
    (StatusCode::OK, "Deleted".to_string())
}

async fn settings_page_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if session.as_ref().map(|s| s.role.as_str()) != Some("Admin") {
        return Html("<h1>Access Denied: Admins Only</h1>".to_string());
    }

    let mut settings = HashMap::new();
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT key, value FROM settings").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let key: String = row.get(0).unwrap();
            let value: String = row.get(1).unwrap();
            settings.insert(key, value);
        }
    }

    let app_name = settings.get("app_name").map(|s| s.as_str()).unwrap_or("AMUD");
    let tagline = settings.get("tagline").map(|s| s.as_str()).unwrap_or("Homelab Operations Cockpit");
    let mut custom_bg_url = settings.get("custom_bg_url").map(|s| s.as_str()).unwrap_or("/static/wallpaper.png");
    if custom_bg_url.is_empty() || custom_bg_url == "https://raw.githubusercontent.com/youssef-boubli/assets/main/dashboard-bg.jpg" {
        custom_bg_url = "/static/wallpaper.png";
    }
    let app_logo = settings.get("app_logo").map(|s| s.as_str()).unwrap_or("");
    let accent_color = settings.get("accent_color").map(|s| s.as_str()).unwrap_or("#cf6427");
    let glass_blur = settings.get("glass_blur_intensity").map(|s| s.as_str()).unwrap_or("16");
    let glass_opacity = settings.get("glass_opacity").map(|s| s.as_str()).unwrap_or("0.45");
    let bento_radius = settings.get("bento_radius").map(|s| s.as_str()).unwrap_or("16");
    let grid_columns = settings
        .get("grid_columns")
        .or_else(|| settings.get("app_grid_columns"))
        .map(|s| s.as_str())
        .unwrap_or("3");
    let overlay_theme = settings.get("overlay_theme").map(|s| s.as_str()).unwrap_or("cyber");
    let custom_overlay_color = settings.get("custom_overlay_color").map(|s| s.as_str()).unwrap_or("#1a1a2e");
    let weather_latitude = settings.get("weather_latitude").map(|s| s.as_str()).unwrap_or("");
    let weather_longitude = settings.get("weather_longitude").map(|s| s.as_str()).unwrap_or("");
    let pve_api_token = settings.get("pve_api_token").map(|s| s.as_str()).unwrap_or("");
    let jellyfin_url = settings.get("jellyfin_url").map(|s| s.as_str()).unwrap_or("");
    let jellyfin_api_key = settings.get("jellyfin_api_key").map(|s| s.as_str()).unwrap_or("");
    let plex_url = settings.get("plex_url").map(|s| s.as_str()).unwrap_or("");
    let plex_token = settings.get("plex_token").map(|s| s.as_str()).unwrap_or("");
    let donate_enabled = settings.get("donate_enabled").map(|s| s.as_str()).unwrap_or("1");

    let bg_url_style = if custom_bg_url.is_empty() { "".to_string() } else { format!("--brand-bg-image: url('{}');", custom_bg_url) };
    let logo_url_style = if app_logo.is_empty() { "".to_string() } else { format!("--brand-logo-url: url('{}');", app_logo) };
    
    let opacity_f: f64 = glass_opacity.parse().unwrap_or(0.45);
    let accent_glow = if accent_color.starts_with('#') && accent_color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&accent_color[1..3], 16),
            u8::from_str_radix(&accent_color[3..5], 16),
            u8::from_str_radix(&accent_color[5..7], 16),
        ) {
            format!("rgba({}, {}, {}, 0.15)", r, g, b)
        } else {
            "rgba(56, 189, 248, 0.15)".to_string()
        }
    } else {
        "rgba(56, 189, 248, 0.15)".to_string()
    };
    
    let overlay_gradient = get_overlay_gradient(overlay_theme, Some(custom_overlay_color));

    let root_css = format!(
        r#"
            {}
            {}
            --brand-title: "{}";
            --brand-slogan: "{}";
            --accent-color: {};
            --accent-glow: {};
            --glass-blur-intensity: {}px;
            --glass-opacity: {};
            --radius-xl: {}px;
            --grid-cols: {};
            --bg-card: rgba(15, 20, 25, {});
            --brand-overlay-gradient: {};
        "#,
        bg_url_style, logo_url_style, app_name, tagline, accent_color, accent_glow, glass_blur, glass_opacity, bento_radius, grid_columns, opacity_f, overlay_gradient
    );

    let settings_tmpl = include_str!("../../ui/templates/settings.html");
    let username = session.as_ref().map(|s| s.username.as_str()).unwrap_or("guest");
    let result = settings_tmpl
        .replace("/* ROOT_CSS */", &root_css)
        .replace("{{app_name}}", app_name)
        .replace("{{tagline}}", tagline)
        .replace("{{custom_bg_url}}", custom_bg_url)
        .replace("{{app_logo}}", app_logo)
        .replace("{{if app_logo}}", if app_logo.is_empty() { "" } else { "" })
        .replace("{{end}}", "")
        .replace("{{accent_color}}", accent_color)
        .replace("{{glass_blur_intensity}}", glass_blur)
        .replace("{{glass_opacity}}", glass_opacity)
        .replace("{{bento_radius}}", bento_radius)
        .replace("{{eq_grid_2}}", if grid_columns == "2" { "selected" } else { "" })
        .replace("{{eq_grid_3}}", if grid_columns == "3" { "selected" } else { "" })
        .replace("{{eq_grid_4}}", if grid_columns == "4" { "selected" } else { "" })
        .replace("{{eq_grid_5}}", if grid_columns == "5" { "selected" } else { "" })
        .replace("{{weather_latitude}}", weather_latitude)
        .replace("{{weather_longitude}}", weather_longitude)
        .replace("{{pve_api_token}}", pve_api_token)
        .replace("{{jellyfin_url}}", jellyfin_url)
        .replace("{{jellyfin_api_key}}", jellyfin_api_key)
        .replace("{{plex_url}}", plex_url)
        .replace("{{plex_token}}", plex_token)
        .replace("{{username}}", username)
        .replace("{{eq_cyber}}", if overlay_theme == "cyber" { "selected" } else { "" })
        .replace("{{eq_aurora}}", if overlay_theme == "aurora" { "selected" } else { "" })
        .replace("{{eq_crimson}}", if overlay_theme == "crimson" { "selected" } else { "" })
        .replace("{{eq_sunset}}", if overlay_theme == "sunset" { "selected" } else { "" })
        .replace("{{eq_obsidian}}", if overlay_theme == "obsidian" { "selected" } else { "" })
        .replace("{{eq_custom}}", if overlay_theme == "custom" { "selected" } else { "" })
        .replace("{{custom_overlay_color}}", custom_overlay_color)
        .replace("{{eq_donate_on}}", if donate_enabled == "1" { "selected" } else { "" })
        .replace("{{eq_donate_off}}", if donate_enabled != "1" { "selected" } else { "" });

    Html(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_container_alerts_transition() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS webhooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                event_types TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO webhooks (name, url, event_types, is_active) VALUES (?, ?, ?, ?)",
            params!["Test WH", "https://discord.com/api/webhooks/test", "container_stopped", 1],
        )
        .unwrap();

        let state = Arc::new(AppState {
            db: Arc::new(Mutex::new(conn)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            latest_telemetry: Arc::new(RwLock::new(AgentTelemetry::default())),
            agent_connected: Arc::new(RwLock::new(false)),
            media_streams: Arc::new(RwLock::new(default_media_streams())),
            app_statuses: Arc::new(RwLock::new(HashMap::new())),
            agent_command_tx: Arc::new(Mutex::new(None)),
            pve_test_response: Arc::new(RwLock::new(None)),
            alert_cooldowns: Arc::new(Mutex::new(HashMap::new())),
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            agent_secret: Arc::new("test-secret".to_string()),
        });

        let old_telemetry = AgentTelemetry {
            cpu_usage: 0,
            ram_usage: 0,
            ram_used_gb: 0.0,
            ram_total_gb: 0.0,
            cpu_temp: 0.0,
            disk_usage: 0,
            disk_used_gb: 0.0,
            disk_total_gb: 0.0,
            network: None,
            lxc_containers: vec![LxcContainer {
                vmid: 100,
                status: "running".to_string(),
                name: "test-lxc".to_string(),
                ..Default::default()
            }],
        };

        let new_telemetry = AgentTelemetry {
            cpu_usage: 0,
            ram_usage: 0,
            ram_used_gb: 0.0,
            ram_total_gb: 0.0,
            cpu_temp: 0.0,
            disk_usage: 0,
            disk_used_gb: 0.0,
            disk_total_gb: 0.0,
            network: None,
            lxc_containers: vec![LxcContainer {
                vmid: 100,
                status: "stopped".to_string(),
                name: "test-lxc".to_string(),
                ..Default::default()
            }],
        };

        // Trigger alert check
        check_container_alerts(&old_telemetry, &new_telemetry, &state);

        // Verify that it added the cooldown key (lxc:test-lxc) to alert_cooldowns map
        let cooldowns = state.alert_cooldowns.lock().unwrap();
        assert!(cooldowns.contains_key("lxc:test-lxc"));
    }

    #[tokio::test]
    async fn test_check_container_alerts_no_change() {

        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS webhooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                event_types TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )
        .unwrap();

        let state = Arc::new(AppState {
            db: Arc::new(Mutex::new(conn)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            latest_telemetry: Arc::new(RwLock::new(AgentTelemetry::default())),
            agent_connected: Arc::new(RwLock::new(false)),
            media_streams: Arc::new(RwLock::new(default_media_streams())),
            app_statuses: Arc::new(RwLock::new(HashMap::new())),
            agent_command_tx: Arc::new(Mutex::new(None)),
            pve_test_response: Arc::new(RwLock::new(None)),
            alert_cooldowns: Arc::new(Mutex::new(HashMap::new())),
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            agent_secret: Arc::new("test-secret".to_string()),
        });

        let old_telemetry = AgentTelemetry {
            lxc_containers: vec![LxcContainer {
                vmid: 100,
                status: "running".to_string(),
                name: "test-lxc".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let new_telemetry = AgentTelemetry {
            lxc_containers: vec![LxcContainer {
                vmid: 100,
                status: "running".to_string(),
                name: "test-lxc".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        check_container_alerts(&old_telemetry, &new_telemetry, &state);

        let cooldowns = state.alert_cooldowns.lock().unwrap();
        assert!(!cooldowns.contains_key("lxc:test-lxc"));
    }
}


