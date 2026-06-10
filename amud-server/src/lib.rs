pub mod agent;
pub mod apps;
pub mod audit;
pub mod auth;
pub mod db;
pub mod handlers;
pub mod media;
pub mod models;
pub mod security;
pub mod settings;
pub mod templates;
pub mod webhooks;

use agent::start_agent_listener;
use auth::{generate_bootstrap_password, hash_password, resolve_agent_secret, security_headers, start_session_cleanup};
use db::refresh_settings_cache;
use handlers::*;
use media::start_media_poller;
use models::{AgentTelemetry, AppState, Session};
use settings::get_default_settings;
use webhooks::start_status_poller;

use axum::{middleware, routing::{get, post}, Router};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex, RwLock};
use tokio::net::TcpListener as TokioTcpListener;

pub async fn run() {
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

conn.execute(
    "CREATE TABLE IF NOT EXISTS audit_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        username TEXT NOT NULL,
        action TEXT NOT NULL,
        target TEXT NOT NULL,
        details TEXT NOT NULL DEFAULT '',
        client_ip TEXT NOT NULL DEFAULT ''
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
        let bootstrap_password = generate_bootstrap_password();
        println!("Seeding initial admin account...");
        let admin_hash = hash_password(&bootstrap_password);
        conn.execute(
            "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
            params!["admin", admin_hash, "Admin"],
        )
        .ok();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('admin_must_change_password', '1')
             ON CONFLICT(key) DO UPDATE SET value = '1'",
            [],
        )
        .ok();
        eprintln!("================================================================");
        eprintln!(" AMUD INITIAL ADMIN CREDENTIALS (save this now — shown once)");
        eprintln!("   Username: admin");
        eprintln!("   Password: {}", bootstrap_password);
        eprintln!(" You will be prompted to change this password after first login.");
        eprintln!("================================================================");
    }
}

let agent_secret = resolve_agent_secret(&conn);
if agent_secret.trim().is_empty() {
    eprintln!("FATAL: AMUD_AGENT_SECRET / agent_shared_secret is not configured.");
    std::process::exit(1);
}

let shared_db = Arc::new(Mutex::new(conn));
let sessions = Arc::new(RwLock::new(HashMap::<String, Session>::new()));
let latest_telemetry = Arc::new(RwLock::new(AgentTelemetry::default()));
let agent_connected = Arc::new(RwLock::new(false));
let media_streams = Arc::new(RwLock::new(HashMap::new()));
let app_statuses = Arc::new(RwLock::new(HashMap::new()));
let agent_command_tx = Arc::new(Mutex::new(None));
let pve_test_response = Arc::new(RwLock::new(None));
let action_results = Arc::new(RwLock::new(HashMap::new()));
let settings_cache = Arc::new(RwLock::new(HashMap::new()));
{
    let db = shared_db.lock().unwrap();
    refresh_settings_cache(&db, &settings_cache);
}

let state = Arc::new(AppState {
    db: shared_db.clone(),
    sessions: sessions.clone(),
    latest_telemetry: latest_telemetry.clone(),
    agent_connected: agent_connected.clone(),
    media_streams: media_streams.clone(),
    app_statuses: app_statuses.clone(),
    agent_command_tx: agent_command_tx.clone(),
    pve_test_response: pve_test_response.clone(),
    action_results: action_results.clone(),
    settings_cache: settings_cache.clone(),
    alert_cooldowns: Arc::new(Mutex::new(HashMap::new())),
    login_attempts: Arc::new(Mutex::new(HashMap::new())),
    api_rate_limits: Arc::new(Mutex::new(HashMap::new())),
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
    .route("/logout", get(logout_handler).post(logout_handler))
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
    .route("/api/audit", get(list_audit_handler))
    .route(
        "/api/users",
        get(list_users_handler).post(add_user_handler),
    )
    .route("/api/users/edit", post(edit_user_handler))
    .route("/api/users/delete", post(delete_user_handler))
    .route("/uploads/:filename", get(serve_upload_handler))
    .nest_service("/static", tower_http::services::ServeDir::new("ui/static"))
    .layer(middleware::from_fn(security_headers))
    .with_state(state);

let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
let addr = format!("0.0.0.0:{}", port);
println!("AMUD Web Server listening online on http://{}", addr);

let listener = TokioTcpListener::bind(&addr).await.unwrap();
axum::serve(listener, app).await.unwrap();
}

