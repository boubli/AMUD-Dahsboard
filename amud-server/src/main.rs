use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        Multipart, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form,
    Router,
};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path as FilePath;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::net::{TcpListener as TokioTcpListener, UnixListener as TokioUnixListener};

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
}

#[derive(Serialize, Clone)]
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

#[derive(Serialize, Clone)]
struct AppMetrics {
    status: String,
    metrics: HashMap<String, String>,
}

#[derive(Serialize, Clone)]
struct FullTelemetry {
    system: AgentTelemetry,
    network: NetworkTelemetry,
    streams: HashMap<String, MediaStream>,
    apps: HashMap<String, AppMetrics>,
}

// Global App State
#[allow(dead_code)]
struct AppState {
    db: Arc<Mutex<Connection>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    latest_telemetry: Arc<RwLock<AgentTelemetry>>,
    agent_connected: Arc<RwLock<bool>>,
    plex_progress: Arc<RwLock<f64>>,
}

// Global default settings
fn get_default_settings() -> HashMap<&'static str, &'static str> {
    let mut s = HashMap::new();
    s.insert("app_name", "AMUD");
    s.insert("tagline", "Homelab Operations Cockpit");
    s.insert("accent_color", "#38bdf8");
    s.insert("custom_bg_url", "/static/wallpaper.png");
    s.insert("app_logo", "");
    s.insert("glass_blur_intensity", "16");
    s.insert("glass_opacity", "0.45");
    s.insert("bento_radius", "16");
    s.insert("weather_info", "13.1°F Clear");
    s.into()
}

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

    // Check settings count
    {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM settings").unwrap();
        let count: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
        if count == 0 {
            println!("Seeding default settings...");
            for (key, val) in get_default_settings() {
                conn.execute(
                    "INSERT INTO settings (key, value) VALUES (?, ?)",
                    params![key, val],
                )
                .ok();
            }
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

    let shared_db = Arc::new(Mutex::new(conn));
    let sessions = Arc::new(RwLock::new(HashMap::<String, Session>::new()));
    let latest_telemetry = Arc::new(RwLock::new(AgentTelemetry::default()));
    let agent_connected = Arc::new(RwLock::new(false));
    let plex_progress = Arc::new(RwLock::new(66.2));

    let state = Arc::new(AppState {
        db: shared_db.clone(),
        sessions: sessions.clone(),
        latest_telemetry: latest_telemetry.clone(),
        agent_connected: agent_connected.clone(),
        plex_progress: plex_progress.clone(),
    });

    // Start Host Agent listener (Background task)
    start_agent_listener(latest_telemetry, agent_connected);

    // Start Plex Playback Simulator (Background task)
    start_plex_simulator(plex_progress);

    // Set up Axum Router
    let app = Router::new()
        .route("/", get(dashboard_handler))
        .route("/login", get(login_page).post(login_handler))
        .route("/logout", get(logout_handler))
        .route("/ws", get(ws_handler))
        .route("/admin/settings", post(settings_handler))
        .route("/admin/upload", post(upload_handler))
        .route("/apps", post(add_app_handler))
        .route("/apps/delete", post(delete_app_handler))
        .nest_service("/uploads", tower_http::services::ServeDir::new("data/uploads"))
        .nest_service("/static", tower_http::services::ServeDir::new("ui/static"))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("AMUD Web Server listening online on http://{}", addr);

    let listener = TokioTcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Password hashing helper
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

// Get User session helper
fn get_session(headers: &HeaderMap, sessions: &RwLock<HashMap<String, Session>>) -> Option<Session> {
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
        .and_then(|token| sessions.read().unwrap().get(&token).cloned())
}

// Playback Simulator task
fn start_plex_simulator(plex_progress: Arc<RwLock<f64>>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let mut val = plex_progress.write().unwrap();
            *val += 0.05;
            if *val >= 100.0 {
                *val = 0.0;
            }
        }
    });
}

// Metrics collector listener task
fn start_agent_listener(
    latest_telemetry: Arc<RwLock<AgentTelemetry>>,
    agent_connected: Arc<RwLock<bool>>,
) {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            let socket_path = std::env::var("AMUD_SOCKET_PATH")
                .unwrap_or_else(|_| "/opt/amud/run/amud.sock".to_string());
            run_uds_listener(&socket_path, latest_telemetry.clone(), agent_connected.clone()).await;
        }

        #[cfg(windows)]
        {
            let addr = std::env::var("AMUD_TCP_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8050".to_string());
            run_tcp_listener(&addr, latest_telemetry.clone(), agent_connected.clone()).await;
        }
    });
}

#[cfg(unix)]
async fn run_uds_listener(
    path: &str,
    latest_telemetry: Arc<RwLock<AgentTelemetry>>,
    agent_connected: Arc<RwLock<bool>>,
) {
    let uds_path = if FilePath::new(path).parent().map(|p| p.exists()).unwrap_or(false) {
        path
    } else {
        "/tmp/amud.sock"
    };

    println!("Starting agent listener via UNIX Domain Socket at {}", uds_path);
    fs::remove_file(uds_path).ok();
    
    let listener = match TokioUnixListener::bind(uds_path) {
        Ok(l) => {
            fs::set_permissions(uds_path, std::fs::Permissions::from_mode(0o666)).ok();
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
            *agent_connected.write().unwrap() = true;
            let (reader, _) = stream.into_split();
            let t_clone = latest_telemetry.clone();
            let c_clone = agent_connected.clone();
            
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut reader = tokio::io::BufReader::new(reader);
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break; // EOF
                    }
                    if let Ok(metrics) = serde_json::from_str::<AgentTelemetry>(&line) {
                        *t_clone.write().unwrap() = metrics;
                    }
                    line.clear();
                }
                println!("AMUD-Agent telemetry client disconnected.");
                *c_clone.write().unwrap() = false;
            });
        }
    }
}

// For fallback or cross-compiles
#[cfg(not(unix))]
async fn run_uds_listener(
    _path: &str,
    _latest_telemetry: Arc<RwLock<AgentTelemetry>>,
    _agent_connected: Arc<RwLock<bool>>,
) {}

#[allow(dead_code)]
async fn run_tcp_listener(
    addr: &str,
    latest_telemetry: Arc<RwLock<AgentTelemetry>>,
    agent_connected: Arc<RwLock<bool>>,
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
            *agent_connected.write().unwrap() = true;
            let (reader, _) = stream.into_split();
            let t_clone = latest_telemetry.clone();
            let c_clone = agent_connected.clone();

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut reader = tokio::io::BufReader::new(reader);
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break; // EOF
                    }
                    if let Ok(metrics) = serde_json::from_str::<AgentTelemetry>(&line) {
                        *t_clone.write().unwrap() = metrics;
                    }
                    line.clear();
                }
                println!("AMUD-Agent telemetry client disconnected.");
                *c_clone.write().unwrap() = false;
            });
        }
    }
}

fn get_overlay_gradient(theme: &str) -> &'static str {
    match theme.to_lowercase().as_str() {
        "aurora" => "linear-gradient(135deg, rgba(4, 15, 15, 0.88) 0%, rgba(6, 24, 20, 0.82) 100%)",
        "crimson" => "linear-gradient(135deg, rgba(18, 8, 8, 0.88) 0%, rgba(12, 10, 15, 0.82) 100%)",
        "obsidian" => "linear-gradient(135deg, rgba(10, 10, 12, 0.92) 0%, rgba(15, 15, 18, 0.88) 100%)",
        "sunset" => "linear-gradient(135deg, rgba(20, 8, 12, 0.88) 0%, rgba(8, 10, 20, 0.82) 100%)",
        _ => "linear-gradient(135deg, rgba(8, 10, 18, 0.85) 0%, rgba(15, 10, 25, 0.8) 100%)", // default cyber
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
    let app_name = settings.get("app_name").map(|s| s.as_str()).unwrap_or("AMUD");
    let tagline = settings.get("tagline").map(|s| s.as_str()).unwrap_or("Homelab Operations Cockpit");
    let mut custom_bg_url = settings.get("custom_bg_url").map(|s| s.as_str()).unwrap_or("/static/wallpaper.png");
    if custom_bg_url.is_empty() || custom_bg_url == "https://raw.githubusercontent.com/youssef-boubli/assets/main/dashboard-bg.jpg" {
        custom_bg_url = "/static/wallpaper.png";
    }
    let app_logo = settings.get("app_logo").map(|s| s.as_str()).unwrap_or("");
    let accent_color = settings.get("accent_color").map(|s| s.as_str()).unwrap_or("#38bdf8");
    let glass_blur = settings.get("glass_blur_intensity").map(|s| s.as_str()).unwrap_or("16");
    let glass_opacity = settings.get("glass_opacity").map(|s| s.as_str()).unwrap_or("0.45");
    let bento_radius = settings.get("bento_radius").map(|s| s.as_str()).unwrap_or("16");
    let weather_info = settings.get("weather_info").map(|s| s.as_str()).unwrap_or("13.1°F Clear");
    let overlay_theme = settings.get("overlay_theme").map(|s| s.as_str()).unwrap_or("cyber");
    let is_admin = session.as_ref().map(|s| s.role == "Admin").unwrap_or(false);

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
        // Group by columns or render in a beautiful 3-column grid
        let mut cols = vec![String::new(), String::new(), String::new()];
        for (i, app) in apps.iter().enumerate() {
            let col_idx = i % 3;
            
            // Resolve Built-in Brand Logo
            let brand_logo = match app.icon.to_lowercase().as_str() {
                "plex" => "/static/logos/plex.svg",
                "jellyfin" => "/static/logos/jellyfin.svg",
                "proxmox" => "/static/logos/proxmox.svg",
                "portainer" => "/static/logos/portainer.svg",
                "home-assistant" | "homeassistant" => "/static/logos/home-assistant.svg",
                "nextcloud" => "/static/logos/nextcloud.svg",
                "adguard" | "adguard-home" => "/static/logos/adguard-home.svg",
                "pihole" | "pi-hole" => "/static/logos/pi-hole.svg",
                "sonarr" => "/static/logos/sonarr.svg",
                "radarr" => "/static/logos/radarr.svg",
                "qbittorrent" => "/static/logos/qbittorrent.svg",
                "transmission" => "/static/logos/transmission.svg",
                "overseerr" => "/static/logos/overseerr.svg",
                "truenas" => "/static/logos/truenas.svg",
                "casaos" => "/static/logos/casaos.svg",
                _ => {
                    if app.icon.starts_with("http") || app.icon.starts_with("/") {
                        &app.icon
                    } else {
                        "/static/fallback.svg"
                    }
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
                r#"<span class="status-badge">ACTIVE</span>"#
            };

            // Build Sub-Metrics Grid
            let sub_metrics;
            let name_lower = app.name.to_lowercase();
            if name_lower.contains("radarr") {
                sub_metrics = r#"
                <div class="nested-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value">21</span>
                        <span class="metric-label">Wanted</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">56</span>
                        <span class="metric-label">Movies</span>
                    </div>
                </div>"#.to_string();
            } else if name_lower.contains("sonarr") {
                sub_metrics = r#"
                <div class="nested-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value">388</span>
                        <span class="metric-label">Wanted</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">11</span>
                        <span class="metric-label">Series</span>
                    </div>
                </div>"#.to_string();
            } else if name_lower.contains("overseerr") {
                sub_metrics = r#"
                <div class="nested-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value">0</span>
                        <span class="metric-label">Pending</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">22</span>
                        <span class="metric-label">Available</span>
                    </div>
                </div>"#.to_string();
            } else if name_lower.contains("sabnzbd") {
                sub_metrics = r#"
                <div class="nested-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value">0 B/s</span>
                        <span class="metric-label">Rate</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">0</span>
                        <span class="metric-label">Queue</span>
                    </div>
                </div>"#.to_string();
            } else if name_lower.contains("deluge") {
                sub_metrics = r#"
                <div class="nested-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value">0 B/s</span>
                        <span class="metric-label">Download</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">211 kB/s</span>
                        <span class="metric-label">Upload</span>
                    </div>
                </div>"#.to_string();
            } else if name_lower.contains("prowlarr") {
                sub_metrics = r#"
                <div class="nested-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value">312</span>
                        <span class="metric-label">Grabs</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">890</span>
                        <span class="metric-label">Queries</span>
                    </div>
                </div>"#.to_string();
            } else if name_lower.contains("proxmox") {
                sub_metrics = r#"
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
                </div>"#.to_string();
            } else if name_lower.contains("truenas") {
                sub_metrics = r#"
                <div class="nested-metrics-grid cols-3">
                    <div class="metric-block">
                        <span class="metric-value">0.21</span>
                        <span class="metric-label">Load</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">7 days</span>
                        <span class="metric-label">Uptime</span>
                    </div>
                    <div class="metric-block" style="color: #ef4444;">
                        <span class="metric-value">4</span>
                        <span class="metric-label">Alerts</span>
                    </div>
                </div>"#.to_string();
            } else if name_lower.contains("portainer") {
                sub_metrics = r#"
                <div class="nested-metrics-grid cols-3">
                    <div class="metric-block">
                        <span class="metric-value">23</span>
                        <span class="metric-label">Running</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">0</span>
                        <span class="metric-label">Stopped</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">23</span>
                        <span class="metric-label">Total</span>
                    </div>
                </div>"#.to_string();
            } else if name_lower.contains("nextcloud") {
                sub_metrics = r#"
                <div class="nested-metrics-grid cols-3">
                    <div class="metric-block">
                        <span class="metric-value" style="font-size:0.75rem;">69.6 TB</span>
                        <span class="metric-label">Free</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">1</span>
                        <span class="metric-label">Users</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value" style="font-size:0.75rem;">47,559</span>
                        <span class="metric-label">Files</span>
                    </div>
                </div>"#.to_string();
            } else {
                sub_metrics = r#"
                <div class="nested-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value">Bookmark</span>
                        <span class="metric-label">Type</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">Linked</span>
                        <span class="metric-label">Status</span>
                    </div>
                </div>"#.to_string();
            }

            let delete_btn = if is_admin {
                format!(
                    r#"
                    <form action="/apps/delete" method="POST" style="margin: 0; display: inline-flex; align-items: center;">
                        <input type="hidden" name="id" value="{}">
                        <button type="submit" class="btn-delete-app" title="Delete application">
                            <i data-lucide="trash-2"></i>
                        </button>
                    </form>
                    "#,
                    app.id
                )
            } else {
                "".to_string()
            };

            let card = format!(
                r#"
                <div class="glass-panel app-card">
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
                        <div style="display: flex; align-items: center; gap: 0.5rem;">
                            {}
                            {}
                        </div>
                    </div>
                    {}
                </div>"#,
                app.url, brand_logo, app.name, app.description, status_badge, delete_btn, sub_metrics
            );
            cols[col_idx].push_str(&card);
        }
        
        apps_html = format!(
            r#"
            <div class="bento-column">{}</div>
            <div class="bento-column">{}</div>
            <div class="bento-column">{}</div>"#,
            cols[0], cols[1], cols[2]
        );
    }

    // Auth actions buttons in topbar
    let auth_buttons = if let Some(ref sess) = session {
        let admin_settings_btn = if sess.role == "Admin" {
            r#"
            <button type="button" class="glass-panel btn-admin" @click="addAppModalOpen = true" style="padding:0.5rem 1rem; border-radius:8px; background:rgba(255,255,255,0.02); font-weight:600; cursor:pointer; font-size:0.82rem; display:inline-flex; align-items:center; gap:0.35rem; color:#fff; border:1px solid rgba(255,255,255,0.06);">
                <i data-lucide="plus" style="width:0.95rem; height:0.95rem;"></i> Add App
            </button>
            <button type="button" class="glass-panel btn-admin" @click="drawerOpen = true" style="padding:0.5rem 1rem; border-radius:8px; background:rgba(255,255,255,0.02); font-weight:600; cursor:pointer; font-size:0.82rem; display:inline-flex; align-items:center; gap:0.35rem; color:#fff; border:1px solid rgba(255,255,255,0.06);">
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
    let has_plex = apps.iter().any(|app| app.name.to_lowercase().contains("plex"));
    let has_jellyfin = apps.iter().any(|app| app.name.to_lowercase().contains("jellyfin") || app.name.to_lowercase().contains("emby"));
    
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
                    <span class="stream-status-badge">RUNNING</span>
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
                    <span class="stream-status-badge">RUNNING</span>
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
        
        let cols_class = if has_plex && has_jellyfin { "streams-row" } else { "streams-row single-col" };
        streams_html = format!(
            r#"<section class="{}">{}</section>"#,
            cols_class, cards
        );
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
        let cat_slug = cat.to_lowercase().replace(' ', "-");
        category_tabs_html.push_str(&format!(
            r#"<button class="filter-tab" onclick="filterCategory('{}', this)">{} <span class="filter-count">{}</span></button>"#,
            cat_slug, cat, count
        ));
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
            u8::from_str_radix(&accent_color[5..7], 16)
        ) {
            format!("rgba({}, {}, {}, 0.15)", r, g, b)
        } else {
            "rgba(56, 189, 248, 0.15)".to_string()
        }
    } else {
        "rgba(56, 189, 248, 0.15)".to_string()
    };

    let overlay_gradient = get_overlay_gradient(overlay_theme);

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
        opacity_f,
        overlay_gradient
    );

    // Load templates
    let index_tmpl = include_str!("../../ui/templates/index.html");
    let username = session.as_ref().map(|s| s.username.as_str()).unwrap_or("guest");
    let result = index_tmpl
        .replace("/* ROOT_CSS */", &root_css)
        .replace("{{app_name}}", app_name)
        .replace("{{tagline}}", tagline)
        .replace("{{custom_bg_url}}", custom_bg_url)
        .replace("{{app_logo}}", app_logo)
        .replace("{{accent_color}}", accent_color)
        .replace("{{glass_blur_intensity}}", glass_blur)
        .replace("{{glass_opacity}}", glass_opacity)
        .replace("{{bento_radius}}", bento_radius)
        .replace("{{weather_info}}", weather_info)
        .replace("<!-- APPS_GRID -->", &apps_html)
        .replace("<!-- STREAMS_ROW -->", &streams_html)
        .replace("<!-- CATEGORY_TABS -->", &category_tabs_html)
        .replace("<!-- AUTH_BUTTONS -->", &auth_buttons)
        .replace("{{username}}", username)
        .replace("{{eq_cyber}}", if overlay_theme == "cyber" { "selected" } else { "" })
        .replace("{{eq_aurora}}", if overlay_theme == "aurora" { "selected" } else { "" })
        .replace("{{eq_crimson}}", if overlay_theme == "crimson" { "selected" } else { "" })
        .replace("{{eq_sunset}}", if overlay_theme == "sunset" { "selected" } else { "" })
        .replace("{{eq_obsidian}}", if overlay_theme == "obsidian" { "selected" } else { "" })
        .replace("{{is_admin}}", if is_admin { "true" } else { "false" });

    Html(result)
}

async fn login_page(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
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
    
    let mut custom_bg_url = settings.get("custom_bg_url").map(|s| s.as_str()).unwrap_or("/static/wallpaper.png");
    if custom_bg_url.is_empty() || custom_bg_url == "https://raw.githubusercontent.com/youssef-boubli/assets/main/dashboard-bg.jpg" {
        custom_bg_url = "/static/wallpaper.png";
    }
    let app_logo = settings.get("app_logo").map(|s| s.as_str()).unwrap_or("");
    let app_name = settings.get("app_name").map(|s| s.as_str()).unwrap_or("AMUD");
    let accent_color = settings.get("accent_color").map(|s| s.as_str()).unwrap_or("#38bdf8");
    let glass_blur = settings.get("glass_blur_intensity").map(|s| s.as_str()).unwrap_or("16");
    let glass_opacity = settings.get("glass_opacity").map(|s| s.as_str()).unwrap_or("0.45");
    let bento_radius = settings.get("bento_radius").map(|s| s.as_str()).unwrap_or("16");
    let overlay_theme = settings.get("overlay_theme").map(|s| s.as_str()).unwrap_or("cyber");

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
            u8::from_str_radix(&accent_color[5..7], 16)
        ) {
            format!("rgba({}, {}, {}, 0.15)", r, g, b)
        } else {
            "rgba(56, 189, 248, 0.15)".to_string()
        }
    } else {
        "rgba(56, 189, 248, 0.15)".to_string()
    };

    let overlay_gradient = get_overlay_gradient(overlay_theme);

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
    let username = form.get("username").cloned().unwrap_or_default();
    let password = form.get("password").cloned().unwrap_or_default();

    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT password_hash, role FROM users WHERE username = ?")
        .unwrap();
    
    let hashed = hash_password(&password);
    let auth_res = stmt.query_row(params![username], |row| {
        let pwhash: String = row.get(0).unwrap();
        let role: String = row.get(1).unwrap();
        Ok((pwhash == hashed, role))
    });

    if let Ok((true, role)) = auth_res {
        let token = format!(
            "{:x}",
            Sha256::digest(format!("{}{}", username, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()).as_bytes())
        );
        
        state.sessions.write().unwrap().insert(
            token.clone(),
            Session {
                username,
                role,
            },
        );

        let cookie = format!(
            "amud_session={}; Path=/; Max-Age=86400; HttpOnly",
            token
        );
        
        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::SET_COOKIE, cookie)
            .header(header::LOCATION, "/")
            .body(axum::body::Body::empty())
            .unwrap()
    } else {
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
        .header(header::SET_COOKIE, "amud_session=; Path=/; Max-Age=0")
        .header(header::LOCATION, "/")
        .body(axum::body::Body::empty())
        .unwrap()
}

// WS upgrades handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws_session(socket, state))
}

async fn handle_ws_session(mut socket: WebSocket, state: Arc<AppState>) {
    let rx_stream = state.latest_telemetry.clone();
    let p_progress = state.plex_progress.clone();

    loop {
        // Stream telemetry packet every 3 seconds
        let system_metrics = rx_stream.read().unwrap().clone();
        
        // Build mock network info matching image_10503b
        let network = NetworkTelemetry {
            internal_tx: "9 kbit/s".to_string(),
            internal_rx: "2 kbit/s".to_string(),
            external_tx: "540 kbit/s".to_string(),
            external_rx: "50 kbit/s".to_string(),
        };

        // Media simulations
        let progress = *p_progress.read().unwrap();
        let plex = MediaStream {
            status: "RUNNING".to_string(),
            active: true,
            title: "Suits - Pilot".to_string(),
            current_time: "47:59".to_string(),
            total_time: "01:12:23".to_string(),
            progress_percent: progress,
        };

        let emby = MediaStream {
            status: "RUNNING".to_string(),
            active: false,
            title: "No Active Streams".to_string(),
            current_time: "".to_string(),
            total_time: "".to_string(),
            progress_percent: 0.0,
        };

        let mut streams = HashMap::new();
        streams.insert("plex".to_string(), plex);
        streams.insert("emby".to_string(), emby);

        // App nested metrics payload
        let mut apps = HashMap::new();
        
        let mut radarr_metrics = HashMap::new();
        radarr_metrics.insert("WANTED".to_string(), "21".to_string());
        radarr_metrics.insert("MOVIES".to_string(), "56".to_string());
        apps.insert("radarr".to_string(), AppMetrics { status: "RUNNING".to_string(), metrics: radarr_metrics });

        let mut sonarr_metrics = HashMap::new();
        sonarr_metrics.insert("WANTED".to_string(), "388".to_string());
        sonarr_metrics.insert("SERIES".to_string(), "11".to_string());
        apps.insert("sonarr".to_string(), AppMetrics { status: "RUNNING".to_string(), metrics: sonarr_metrics });

        let mut overseerr_metrics = HashMap::new();
        overseerr_metrics.insert("PENDING".to_string(), "0".to_string());
        overseerr_metrics.insert("AVAILABLE".to_string(), "22".to_string());
        apps.insert("overseerr".to_string(), AppMetrics { status: "RUNNING".to_string(), metrics: overseerr_metrics });

        let mut sab_metrics = HashMap::new();
        sab_metrics.insert("RATE".to_string(), "0 B/s".to_string());
        sab_metrics.insert("QUEUE".to_string(), "0".to_string());
        apps.insert("sabnzbd".to_string(), AppMetrics { status: "RUNNING".to_string(), metrics: sab_metrics });

        let mut deluge_metrics = HashMap::new();
        deluge_metrics.insert("DOWNLOAD".to_string(), "0 B/s".to_string());
        deluge_metrics.insert("UPLOAD".to_string(), "211 kB/s".to_string());
        apps.insert("deluge".to_string(), AppMetrics { status: "RUNNING".to_string(), metrics: deluge_metrics });

        let mut prowlarr_metrics = HashMap::new();
        prowlarr_metrics.insert("GRABS".to_string(), "312".to_string());
        prowlarr_metrics.insert("QUERIES".to_string(), "890".to_string());
        apps.insert("prowlarr".to_string(), AppMetrics { status: "RUNNING".to_string(), metrics: prowlarr_metrics });

        let payload = FullTelemetry {
            system: system_metrics,
            network,
            streams,
            apps,
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
        for (key, val) in form {
            db.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = ?2",
                params![key, val],
            )
            .ok();
        }
    }
    Redirect::to("/")
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
        let url = form.get("url").cloned().unwrap_or_default();
        let icon = form.get("icon").cloned().unwrap_or_default();
        let category = form.get("category").cloned().unwrap_or_else(|| "General".to_string());
        let node_tag = form.get("node_tag").cloned().unwrap_or_else(|| "Local".to_string());
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
                db.execute("DELETE FROM apps WHERE id = ?", params![id]).ok();
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
            
            if ext != "png" && ext != "jpg" && ext != "jpeg" && ext != "svg" && ext != "ico" && ext != "gif" {
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

            if bytes.len() > 2 * 1024 * 1024 {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from("File size exceeds 2MB limit"))
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
            .body(axum::body::Body::from(format!(r#"{{"url":"{}"}}"#, url_path)))
            .unwrap()
    }
}
