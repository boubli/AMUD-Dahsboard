use super::imports::*;
use rusqlite::Connection;
use std::sync::Mutex;

fn count_table(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0)).unwrap_or(0)
}

fn inspect_backup_bytes(bytes: &[u8]) -> Result<serde_json::Value, &'static str> {
    if bytes.len() < 16 || &bytes[0..16] != b"SQLite format 3\0" {
        return Err("Invalid SQLite database file");
    }
    let temp = std::env::temp_dir().join(format!("amud-val-{}.db", std::process::id()));
    std::fs::write(&temp, bytes).map_err(|_| "Failed to read upload")?;
    let conn = Connection::open(&temp).map_err(|_| "Invalid or corrupt database")?;
    let stats = serde_json::json!({
        "valid": true,
        "apps": count_table(&conn, "SELECT COUNT(*) FROM apps"),
        "users": count_table(&conn, "SELECT COUNT(*) FROM users"),
        "webhooks": count_table(&conn, "SELECT COUNT(*) FROM webhooks"),
        "categories": count_table(&conn, "SELECT COUNT(*) FROM categories"),
        "rss_feeds": count_table(&conn, "SELECT COUNT(*) FROM apps WHERE integration_type = 'rss'"),
    });
    let _ = std::fs::remove_file(&temp);
    Ok(stats)
}

fn db_path() -> String {
    std::env::var("DB_PATH").unwrap_or_else(|_| "data/amud.db".to_string())
}

fn remove_wal_sidecars(db_path: &str) {
    let _ = std::fs::remove_file(format!("{db_path}-wal"));
    let _ = std::fs::remove_file(format!("{db_path}-shm"));
}

async fn wal_checkpoint(db: Arc<Mutex<Connection>>) {
    with_db(db, |conn| {
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    })
    .await;
}

pub async fn list_audit_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }

    let result = with_db(state.db.clone(), |db| list_recent_audit(db, 200)).await;
    match result {
        Ok(entries) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()),
            ))
            .unwrap(),
        Err(message) => Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({ "error": message }).to_string(),
            ))
            .unwrap(),
    }
}

pub async fn export_backup_handler(
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
    let admin_user = get_session(&headers, &state.sessions)
        .map(|s| s.username)
        .unwrap_or_else(|| "admin".to_string());
    let headers = headers.clone();
    with_db(state.db.clone(), move |db| {
        record_audit_blocking(
            db,
            &headers,
            &admin_user,
            "backup_export",
            "amud.db",
            "database download",
        );
        let now = chrono::Utc::now().to_rfc3339();
        db.execute(
            "INSERT INTO settings (key, value) VALUES ('last_backup_export_at', ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![now],
        )
        .ok();
    })
    .await;
    wal_checkpoint(state.db.clone()).await;
    let path = db_path();
    let data = std::fs::read(&path).unwrap_or_default();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"amud.db\"",
        )
        .body(Body::from(data))
        .unwrap()
}

pub async fn validate_backup_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return (*resp).into_response();
    }

    let mut db_data = None;
    let mut csrf = String::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "csrf_token" {
            if let Ok(text) = field.text().await {
                csrf = text;
            }
        } else if name == "db_file" {
            if let Ok(bytes) = field.bytes().await {
                db_data = Some(bytes);
            }
        }
    }

    let mut form_map = HashMap::new();
    form_map.insert("csrf_token".to_string(), csrf);
    if !validate_csrf(&headers, &state.sessions, Some(&Form(form_map))) {
        return csrf_forbidden_response().into_response();
    }

    let Some(bytes) = db_data else {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "No database file uploaded"}),
        )
        .into_response();
    };

    match inspect_backup_bytes(&bytes) {
        Ok(stats) => api_json(StatusCode::OK, stats).into_response(),
        Err(message) => api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"valid": false, "error": message}),
        )
        .into_response(),
    }
}

pub async fn import_backup_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return (*resp).into_response();
    }

    let mut db_data = None;
    let mut csrf = String::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "csrf_token" {
            if let Ok(text) = field.text().await {
                csrf = text;
            }
        } else if name == "db_file" {
            if let Ok(bytes) = field.bytes().await {
                db_data = Some(bytes);
            }
        }
    }

    let mut form_map = HashMap::new();
    form_map.insert("csrf_token".to_string(), csrf);
    if !validate_csrf(&headers, &state.sessions, Some(&Form(form_map))) {
        return csrf_forbidden_response().into_response();
    }

    let Some(bytes) = db_data else {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "No database file uploaded"}),
        )
        .into_response();
    };

    let stats = match inspect_backup_bytes(&bytes) {
        Ok(s) => s,
        Err(message) => {
            return api_json(
                StatusCode::BAD_REQUEST,
                serde_json::json!({"error": message}),
            )
            .into_response();
        }
    };

    let admin_user = get_session(&headers, &state.sessions)
        .map(|s| s.username)
        .unwrap_or_else(|| "admin".to_string());
    let headers = headers.clone();
    let audit_details = format!(
        "apps={}, users={}, webhooks={}",
        stats.get("apps").and_then(|v| v.as_i64()).unwrap_or(0),
        stats.get("users").and_then(|v| v.as_i64()).unwrap_or(0),
        stats.get("webhooks").and_then(|v| v.as_i64()).unwrap_or(0),
    );

    let path = db_path();
    let backup_path = format!("{}.bak", path);

    with_db(state.db.clone(), move |db| {
        record_audit_blocking(
            db,
            &headers,
            &admin_user,
            "backup_import",
            "amud.db",
            &audit_details,
        );
    })
    .await;

    wal_checkpoint(state.db.clone()).await;

    if std::path::Path::new(&path).exists() {
        let _ = std::fs::copy(&path, &backup_path);
    }

    if std::fs::write(&path, &bytes).is_err() {
        return api_json(
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": "Failed to write database file"}),
        )
        .into_response();
    }

    remove_wal_sidecars(&path);

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        std::process::exit(0);
    });

    api_json(
        StatusCode::OK,
        serde_json::json!({
            "ok": true,
            "message": "Database restored. AMUD is restarting…"
        }),
    )
    .into_response()
}
