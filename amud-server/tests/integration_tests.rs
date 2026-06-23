use amud_protocol::AgentTelemetry;
use amud_server::build_app_router;
use amud_server::models::{AppState, Session};
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tower::util::ServiceExt; // for oneshot

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "PRAGMA foreign_keys=ON;
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'Guest'
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            icon TEXT DEFAULT '',
            description TEXT DEFAULT '',
            category TEXT DEFAULT 'General',
            node_tag TEXT DEFAULT 'Local',
            mac_address TEXT DEFAULT '',
            integration_type TEXT DEFAULT '',
            api_key TEXT DEFAULT '',
            sort_order INTEGER DEFAULT 0,
            card_span TEXT DEFAULT '1x1'
        );
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            color TEXT DEFAULT '#64748b',
            sort_order INTEGER DEFAULT 0
        );
        INSERT INTO categories (name, color, sort_order) VALUES ('General', '#64748b', 0);
        INSERT INTO categories (name, color, sort_order) VALUES ('Media', '#64748b', 1);
        CREATE TABLE IF NOT EXISTS feed_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            color TEXT DEFAULT '#64748b',
            icon TEXT DEFAULT 'rss',
            sort_order INTEGER DEFAULT 0
        );
        INSERT INTO feed_categories (name, color, icon, sort_order) VALUES ('General', '#64748b', 'rss', 0);
        INSERT INTO feed_categories (name, color, icon, sort_order) VALUES ('Tech', '#8b5cf6', 'cpu', 1);",
    )
    .unwrap();
    conn
}

fn setup_test_state() -> Arc<AppState> {
    let _ = amud_server::secrets::init_secrets_key(":memory:");

    let conn = setup_test_db();
    let shared_db = Arc::new(Mutex::new(conn));
    let sessions = Arc::new(RwLock::new(HashMap::new()));
    let latest_telemetry = Arc::new(RwLock::new(AgentTelemetry::default()));
    let agent_connected = Arc::new(RwLock::new(false));
    let media_streams = Arc::new(RwLock::new(HashMap::new()));
    let app_statuses = Arc::new(RwLock::new(HashMap::new()));
    let agent_command_tx = Arc::new(Mutex::new(None));
    let pve_test_response = Arc::new(RwLock::new(None));
    let action_results = Arc::new(RwLock::new(HashMap::new()));
    let settings_cache = Arc::new(RwLock::new(HashMap::new()));

    let (telemetry_broadcast, _) = tokio::sync::watch::channel(Arc::new(
        amud_server::telemetry_broadcast::WsTelemetryBundle {
            full: "".into(),
            guest_public: "".into(),
            guest_redacted: "".into(),
        },
    ));

    Arc::new(AppState {
        db: shared_db,
        sessions,
        latest_telemetry,
        agent_connected,
        media_streams,
        app_statuses,
        agent_command_tx,
        next_agent_conn_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        pve_test_response,
        action_results,
        settings_cache,
        alert_cooldowns: Arc::new(Mutex::new(HashMap::new())),
        login_attempts: Arc::new(Mutex::new(HashMap::new())),
        api_rate_limits: Arc::new(Mutex::new(HashMap::new())),
        agent_secret: Arc::new("secret-key".to_string()),
        smart_home_telemetry: Arc::new(RwLock::new(Default::default())),
        logo_manifest: Arc::new(HashMap::new()),
        telemetry_broadcast,
    })
}

#[tokio::test]
async fn test_unauthorized_integration_data() {
    // No session and app id 1 does not exist — expect 403 (not an RSS public-access test).
    let state = setup_test_state();
    let app = build_app_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/apps/1/integration")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_unauthorized_integration_action_csrf() {
    let state = setup_test_state();
    let app = build_app_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/apps/1/integration/action")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"action":"disable"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_app_key_blank_edit_preservation() {
    let state = setup_test_state();

    // Seed database with encrypted API key
    let original_raw_key = "super-secret-pihole-key";
    let encrypted_key = amud_server::secrets::encrypt_value(original_raw_key).unwrap();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, category, node_tag, integration_type, api_key) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![1, "Pihole", "http://1.1.1.1", "General", "Local", "pihole", encrypted_key],
        ).unwrap();
    }

    // Insert an Admin session
    let session_token = "admin-session-token-123";
    let admin_session = Session {
        username: "admin".to_string(),
        role: "Admin".to_string(),
        expires_at_epoch: amud_server::auth::now_epoch_secs() + 3600,
        csrf_token: "csrf-token-abc".to_string(),
    };
    state
        .sessions
        .write()
        .unwrap()
        .insert(session_token.to_string(), admin_session);

    let app = build_app_router(state.clone());

    // Submit form with blank API key
    let edit_payload = "id=1&name=PiholeEdited&url=http://1.1.1.1&category=General&node_tag=Local&integration_type=pihole&api_key=&csrf_token=csrf-token-abc";
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/apps/edit")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("amud_session={}", session_token))
                .body(Body::from(edit_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER); // Redirect back to index

    // Verify key in DB is preserved (and still decrypted properly to the original raw value)
    let preserved_key = {
        let db = state.db.lock().unwrap();
        db.query_row("SELECT api_key FROM apps WHERE id = 1", [], |row| {
            row.get::<_, String>(0)
        })
        .unwrap()
    };

    let decrypted = amud_server::secrets::decrypt_value(&preserved_key).unwrap();
    assert_eq!(decrypted, original_raw_key);
}

#[tokio::test]
async fn test_telemetry_redaction_guest() {
    let state = setup_test_state();

    // Seed database with encrypted API key
    let original_raw_key = "super-secret-pihole-key";
    let encrypted_key = amud_server::secrets::encrypt_value(original_raw_key).unwrap();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, category, node_tag, integration_type, api_key) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![1, "Pihole", "http://1.1.1.1", "General", "Local", "pihole", encrypted_key],
        ).unwrap();
    }

    // Insert an Admin session
    let session_token = "admin-session-token-123";
    let admin_session = Session {
        username: "admin".to_string(),
        role: "Admin".to_string(),
        expires_at_epoch: amud_server::auth::now_epoch_secs() + 3600,
        csrf_token: "csrf-token-abc".to_string(),
    };
    state
        .sessions
        .write()
        .unwrap()
        .insert(session_token.to_string(), admin_session);

    let app = build_app_router(state.clone());

    // Request dashboard as admin
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header(header::COOKIE, format!("amud_session={}", session_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 100 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Admin should see the redacted placeholder in JSON, not the plaintext key
    assert!(body_str.contains("Configured — leave blank to keep unchanged"));
    assert!(!body_str.contains(original_raw_key));
}

fn insert_admin_session(state: &Arc<AppState>) -> String {
    let session_token = "admin-session-reorder";
    let admin_session = Session {
        username: "admin".to_string(),
        role: "Admin".to_string(),
        expires_at_epoch: amud_server::auth::now_epoch_secs() + 3600,
        csrf_token: "csrf-reorder-abc".to_string(),
    };
    state
        .sessions
        .write()
        .unwrap()
        .insert(session_token.to_string(), admin_session);
    session_token.to_string()
}

#[tokio::test]
async fn test_reorder_apps_success() {
    let state = setup_test_state();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, sort_order) VALUES (1, 'A', 'http://a', 0)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, sort_order) VALUES (2, 'B', 'http://b', 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, sort_order) VALUES (3, 'C', 'http://c', 2)",
            [],
        )
        .unwrap();
    }

    let session_token = insert_admin_session(&state);
    let app = build_app_router(state.clone());

    let payload = r#"{"ids":[3,1,2],"csrf_token":"csrf-reorder-abc"}"#;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/apps/reorder")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, format!("amud_session={}", session_token))
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let db = state.db.lock().unwrap();
    let order: Vec<(i64, i64)> = db
        .prepare("SELECT id, sort_order FROM apps ORDER BY sort_order ASC")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(order, vec![(3, 0), (1, 1), (2, 2)]);
}

#[tokio::test]
async fn test_reorder_apps_rejects_unknown_id() {
    let state = setup_test_state();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, sort_order) VALUES (1, 'A', 'http://a', 0)",
            [],
        )
        .unwrap();
    }

    let session_token = insert_admin_session(&state);
    let app = build_app_router(state);

    let payload = r#"{"ids":[1,99],"csrf_token":"csrf-reorder-abc"}"#;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/apps/reorder")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, format!("amud_session={}", session_token))
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reorder_apps_forbidden_for_guest() {
    let state = setup_test_state();
    let app = build_app_router(state);

    let payload = r#"{"ids":[1],"csrf_token":"any"}"#;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/apps/reorder")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_settings_save_records_audit_entry() {
    let state = setup_test_state();
    let session_token = insert_admin_session(&state);

    let app = build_app_router(state.clone());
    let payload = "accent_color=%23cf6427&csrf_token=csrf-reorder-abc";
    let save_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/settings")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("amud_session={session_token}"))
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(save_response.status(), StatusCode::SEE_OTHER);

    let audit_app = build_app_router(state);
    let audit_response = audit_app
        .oneshot(
            Request::builder()
                .uri("/api/audit")
                .header(header::COOKIE, format!("amud_session={session_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(audit_response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(audit_response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(
        entries.iter().any(|entry| {
            entry.get("action").and_then(|v| v.as_str()) == Some("settings_update")
        }),
        "expected settings_update audit entry, got: {entries:?}"
    );
}

#[tokio::test]
async fn test_rss_integration_allowed_for_guest() {
    let state = setup_test_state();
    let encrypted_key =
        amud_server::secrets::encrypt_value("https://example.com/feed.xml").unwrap();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, category, node_tag, integration_type, api_key) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![1, "News", "https://example.com", "General", "Local", "rss", encrypted_key],
        )
        .unwrap();
    }

    let app = build_app_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/apps/1/integration")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_rss_integration_forbidden_for_pihole_guest() {
    let state = setup_test_state();
    let encrypted_key = amud_server::secrets::encrypt_value("super-secret-pihole-key").unwrap();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, category, node_tag, integration_type, api_key) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![1, "Pihole", "http://1.1.1.1", "General", "Local", "pihole", encrypted_key],
        )
        .unwrap();
    }

    let app = build_app_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/apps/1/integration")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_rss_url_rejected_on_add() {
    let state = setup_test_state();
    let session_token = insert_admin_session(&state);
    let app = build_app_router(state.clone());

    let add_payload = "name=News&url=https://example.com&category=General&node_tag=Local&integration_type=rss&api_key=http://127.0.0.1/feed&csrf_token=csrf-reorder-abc";
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/apps")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("amud_session={session_token}"))
                .body(Body::from(add_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let count: i64 = {
        let db = state.db.lock().unwrap();
        db.query_row("SELECT COUNT(*) FROM apps", [], |row| row.get(0))
            .unwrap()
    };
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_integration_data_rate_limited() {
    let state = setup_test_state();
    let app = build_app_router(state);

    let mut last_status = StatusCode::OK;
    for _ in 0..31 {
        last_status = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/apps/1/integration")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status();
    }

    assert_eq!(last_status, StatusCode::TOO_MANY_REQUESTS);
}

// ── Phase 1: Feeds page UI (news cards, no infra metrics) ─────────────────

#[tokio::test]
async fn test_feeds_page_renders_feed_cards_without_infra_metrics() {
    let state = setup_test_state();
    let encrypted_key =
        amud_server::secrets::encrypt_value("https://feeds.bbci.co.uk/news/rss.xml").unwrap();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, icon, category, node_tag, integration_type, api_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                1,
                "BBC News",
                "https://www.bbc.com/news",
                "bbc",
                "General",
                "Local",
                "rss",
                encrypted_key
            ],
        )
        .unwrap();
    }

    let app = build_app_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/feeds")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&body_bytes);

    assert!(body.contains("feeds-grid"), "expected feeds grid layout");
    assert!(body.contains("feed-card"), "expected feed-card component");
    assert!(body.contains("rss-feed-list"), "expected headline list");
    assert!(
        body.contains("page-feeds"),
        "expected feeds page body class"
    );
    assert!(
        body.contains("id=\"feed-hero\""),
        "expected featured hero shell"
    );
    assert!(body.contains("initFeedHero"), "expected hero loader script");
    for (idx, _) in body.match_indices("class=\"glass-panel feed-card\"") {
        let snippet = &body[idx..body.len().min(idx + 2500)];
        assert!(
            !snippet.contains("data-lxc-metrics"),
            "feed-card must not include CPU/RAM metrics grid"
        );
        assert!(
            !snippet.contains("status-badge"),
            "feed-card must not include health status badge"
        );
    }
    assert!(
        !body.contains("app-card-header"),
        "feeds page must not use homelab app-card shell"
    );
}

// ── Phase 2: Feed categories + RSS category assignment ────────────────────

#[tokio::test]
async fn test_feed_categories_list_requires_admin() {
    let state = setup_test_state();
    let app = build_app_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/feed-categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_feed_categories_list_returns_seeded_categories() {
    let state = setup_test_state();
    let session_token = insert_admin_session(&state);
    let app = build_app_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/feed-categories")
                .header(header::COOKIE, format!("amud_session={session_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let categories: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(categories.iter().any(|c| c["name"] == "Tech"));
    assert!(categories.iter().any(|c| c["icon"] == "cpu"));
}

#[tokio::test]
async fn test_rss_feed_api_persists_category() {
    let state = setup_test_state();
    let session_token = insert_admin_session(&state);
    let app = build_app_router(state.clone());

    let payload = "name=BBC+News&feed_url=https%3A%2F%2Ffeeds.bbci.co.uk%2Fnews%2Frss.xml&url=https%3A%2F%2Fwww.bbc.com%2Fnews&icon=bbc&category=Tech&csrf_token=csrf-reorder-abc";
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rss-feeds/add")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("amud_session={session_token}"))
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let category: String = {
        let db = state.db.lock().unwrap();
        db.query_row(
            "SELECT category FROM apps WHERE integration_type = 'rss'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    };
    assert_eq!(category, "Tech");

    let icon: String = {
        let db = state.db.lock().unwrap();
        db.query_row(
            "SELECT icon FROM apps WHERE integration_type = 'rss'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    };
    assert_eq!(icon, "bbc");

    let audit_app = build_app_router(state);
    let audit_response = audit_app
        .oneshot(
            Request::builder()
                .uri("/api/audit")
                .header(header::COOKIE, format!("amud_session={session_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(audit_response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    let create_entry = entries
        .iter()
        .find(|e| e.get("action").and_then(|v| v.as_str()) == Some("rss_feed_create"));
    assert!(
        create_entry.is_some(),
        "expected rss_feed_create audit entry"
    );
    let details = create_entry
        .unwrap()
        .get("details")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(
        details.contains("category=Tech"),
        "audit details: {details}"
    );
    assert!(details.contains("icon=bbc"), "audit details: {details}");
}

#[tokio::test]
async fn test_feeds_page_shows_category_tabs() {
    let state = setup_test_state();
    let encrypted_key =
        amud_server::secrets::encrypt_value("https://feeds.bbci.co.uk/news/rss.xml").unwrap();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, icon, category, node_tag, integration_type, api_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                1,
                "BBC News",
                "https://www.bbc.com/news",
                "bbc",
                "Tech",
                "Local",
                "rss",
                encrypted_key
            ],
        )
        .unwrap();
    }

    let app = build_app_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/feeds")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(
        body.contains("feed-filter-tab"),
        "expected feed category tabs"
    );
    assert!(
        body.contains("feed-category-pill"),
        "expected category pill on card"
    );
    assert!(
        body.contains("--tab-accent"),
        "expected category color on feed tabs"
    );
    assert!(
        body.contains("id=\"feed-hero\""),
        "expected featured hero shell"
    );
}

// ── Phase 4: Feed reorder, hero card, tab colors ──────────────────────────

#[tokio::test]
async fn test_rss_feeds_reorder_success() {
    let state = setup_test_state();
    let key_a = amud_server::secrets::encrypt_value("https://example.com/a.xml").unwrap();
    let key_b = amud_server::secrets::encrypt_value("https://example.com/b.xml").unwrap();
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, category, node_tag, integration_type, api_key, sort_order) VALUES (1, 'A', 'https://a', 'General', 'Local', 'rss', ?, 0)",
            rusqlite::params![key_a],
        )
        .unwrap();
        db.execute(
            "INSERT INTO apps (id, name, url, category, node_tag, integration_type, api_key, sort_order) VALUES (2, 'B', 'https://b', 'General', 'Local', 'rss', ?, 1)",
            rusqlite::params![key_b],
        )
        .unwrap();
    }

    let session_token = insert_admin_session(&state);
    let app = build_app_router(state.clone());

    let payload = r#"{"ids":[2,1],"csrf_token":"csrf-reorder-abc"}"#;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rss-feeds/reorder")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, format!("amud_session={}", session_token))
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let db = state.db.lock().unwrap();
    let first_name: String = db
        .query_row(
            "SELECT name FROM apps WHERE integration_type = 'rss' ORDER BY sort_order ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(first_name, "B");
}
