use super::imports::*;

pub async fn list_audit_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }

    let entries = with_db(state.db.clone(), |db| {
        crate::audit::ensure_audit_log_table(db).ok();
        list_recent_audit(db, 200)
    })
    .await;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()),
        ))
        .unwrap()
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
    })
    .await;
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "data/amud.db".to_string());
    let data = std::fs::read(&db_path).unwrap_or_default();
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

    if let Some(bytes) = db_data {
        if bytes.len() < 16 || &bytes[0..16] != b"SQLite format 3\0" {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Invalid SQLite database file"}"#))
                .unwrap()
                .into_response();
        }

        let admin_user = get_session(&headers, &state.sessions)
            .map(|s| s.username)
            .unwrap_or_else(|| "admin".to_string());
        let headers = headers.clone();

        let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "data/amud.db".to_string());
        let backup_path = format!("{}.bak", db_path);

        with_db(state.db.clone(), move |db| {
            record_audit_blocking(
                db,
                &headers,
                &admin_user,
                "backup_import",
                "amud.db",
                "database restore initiated",
            );
        })
        .await;

        if std::path::Path::new(&db_path).exists() {
            let _ = std::fs::copy(&db_path, &backup_path);
        }

        if std::fs::write(&db_path, &bytes).is_err() {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Failed to write database file"}"#))
                .unwrap()
                .into_response();
        }

        std::process::exit(0);
    }

    Redirect::to("/admin/settings").into_response()
}
