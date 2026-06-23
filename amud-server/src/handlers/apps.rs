use super::imports::*;

pub async fn add_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    let name = form.get("name").cloned().unwrap_or_default();
    let url = normalize_url(&form.get("url").cloned().unwrap_or_default());
    let icon = form.get("icon").cloned().unwrap_or_default();
    let category_input = form
        .get("category")
        .cloned()
        .unwrap_or_else(|| "General".to_string());
    let node_tag = form
        .get("node_tag")
        .cloned()
        .unwrap_or_else(|| "Local".to_string());
    let description = form.get("description").cloned().unwrap_or_default();
    let mac_address = form.get("mac_address").cloned().unwrap_or_default();
    let integration_type = form.get("integration_type").cloned().unwrap_or_default();
    let api_key = form.get("api_key").cloned().unwrap_or_default();
    let rss_key_ok = rss_feed_api_key_valid(&integration_type, &api_key);

    if !name.is_empty() && !url.is_empty() && rss_key_ok {
        let encrypted_api_key = encrypt_integration_api_key(&integration_type, &api_key);
        let admin_user = session
            .as_ref()
            .map(|s| s.username.clone())
            .unwrap_or_default();
        let headers = headers.clone();
        let card_span = form
            .get("card_span")
            .map(|s| sanitize_card_span(s))
            .unwrap_or_else(|| "1x1".to_string());
        with_db(state.db.clone(), move |db| {
            let category = crate::db::resolve_app_category(db, &category_input);
            let sort_order = crate::db::next_app_sort_order(db);
            if db
                .execute(
                    "INSERT INTO apps (name, url, icon, description, category, node_tag, mac_address, integration_type, api_key, sort_order, card_span) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![name, url, icon, description, category, node_tag, mac_address, integration_type, encrypted_api_key, sort_order, card_span],
                )
                .is_ok()
            {
                record_audit_blocking(db, &headers, &admin_user, "app_create", &name, &url);
            }
        })
        .await;
    }
    Redirect::to("/").into_response()
}

pub async fn delete_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let admin_user = session
                .as_ref()
                .map(|s| s.username.clone())
                .unwrap_or_default();
            let headers = headers.clone();
            with_db(state.db.clone(), move |db| {
                let app_name: String = db
                    .query_row("SELECT name FROM apps WHERE id = ?", params![id], |row| {
                        row.get(0)
                    })
                    .unwrap_or_else(|_| format!("id:{id}"));
                if db
                    .execute("DELETE FROM apps WHERE id = ?", params![id])
                    .is_ok()
                {
                    record_audit_blocking(
                        db,
                        &headers,
                        &admin_user,
                        "app_delete",
                        &app_name,
                        "removed",
                    );
                }
            })
            .await;
        }
    }
    Redirect::to("/").into_response()
}

pub async fn edit_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let name = form.get("name").cloned().unwrap_or_default();
            let url = normalize_url(&form.get("url").cloned().unwrap_or_default());
            let icon = form.get("icon").cloned().unwrap_or_default();
            let category_input = form
                .get("category")
                .cloned()
                .unwrap_or_else(|| "General".to_string());
            let node_tag = form
                .get("node_tag")
                .cloned()
                .unwrap_or_else(|| "Local".to_string());
            let description = form.get("description").cloned().unwrap_or_default();
            let mac_address = form.get("mac_address").cloned().unwrap_or_default();
            let integration_type = form.get("integration_type").cloned().unwrap_or_default();
            let api_key = form.get("api_key").cloned().unwrap_or_default();
            let rss_key_ok = rss_feed_api_key_valid(&integration_type, &api_key);

            if !name.is_empty() && !url.is_empty() && rss_key_ok {
                let admin_user = session
                    .as_ref()
                    .map(|s| s.username.clone())
                    .unwrap_or_default();
                let headers = headers.clone();
                let card_span = form
                    .get("card_span")
                    .map(|s| sanitize_card_span(s))
                    .unwrap_or_else(|| "1x1".to_string());
                with_db(state.db.clone(), move |db| {
                    let category = crate::db::resolve_app_category(db, &category_input);
                    let final_api_key = if api_key.trim().is_empty() || api_key == "Configured — leave blank to keep unchanged" {
                        db.query_row("SELECT api_key FROM apps WHERE id = ?", params![id], |row| {
                            row.get::<_, String>(0)
                        }).unwrap_or_default()
                    } else {
                        encrypt_integration_api_key(&integration_type, &api_key)
                    };
                    if db
                        .execute(
                            "UPDATE apps SET name = ?, url = ?, icon = ?, description = ?, category = ?, node_tag = ?, mac_address = ?, integration_type = ?, api_key = ?, card_span = ? WHERE id = ?",
                            params![name, url, icon, description, category, node_tag, mac_address, integration_type, final_api_key, card_span, id],
                        )
                        .is_ok()
                    {
                        record_audit_blocking(
                            db,
                            &headers,
                            &admin_user,
                            "app_update",
                            &name,
                            &url,
                        );
                    }
                })
                .await;
            }
        }
    }
    Redirect::to("/").into_response()
}

fn parse_mac(mac: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let parts: Vec<&str> = mac.split(&[':', '-'][..]).collect();
    if parts.len() == 6 {
        for p in parts {
            if let Ok(b) = u8::from_str_radix(p, 16) {
                bytes.push(b);
            } else {
                return None;
            }
        }
        return Some(bytes);
    }
    None
}

pub async fn upload_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "upload", 10, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = session
        .as_ref()
        .filter(|s| s.role == "Admin")
        .map(|s| s.username.clone());
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Forbidden"))
            .unwrap();
    }
    if !validate_csrf(&headers, &state.sessions, None) {
        return csrf_forbidden_response();
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

            if ext != "png" && ext != "jpg" && ext != "jpeg" && ext != "ico" && ext != "gif" {
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
        if let Some(user) = admin_user {
            let headers = headers.clone();
            let url_path_audit = url_path.clone();
            with_db(state.db.clone(), move |db| {
                record_audit_blocking(db, &headers, &user, "upload", "image", &url_path_audit);
            })
            .await;
        }
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

pub async fn app_action_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "container_action", 30, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session.as_ref() {
        Some(s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Unauthorized"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let provider = form.get("provider").cloned().unwrap_or_default();
    let id = form.get("id").cloned().unwrap_or_default();
    let action = form.get("action").cloned().unwrap_or_default();

    if provider.is_empty() || id.is_empty() || action.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Missing parameters"}"#))
            .unwrap();
    }

    let action_ok = match provider.as_str() {
        "lxc" => matches!(
            action.as_str(),
            "start" | "stop" | "restart" | "reboot" | "shutdown"
        ),
        "docker" => matches!(action.as_str(), "start" | "stop" | "restart"),
        _ => false,
    };
    if !action_ok {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Invalid provider or action"}"#))
            .unwrap();
    }

    let request_id = generate_session_token();
    let cmd_value = serde_json::json!({
        "provider": provider,
        "id": id,
        "action": action,
        "request_id": request_id
    });
    let mut cmd = match serde_json::to_vec(&cmd_value) {
        Ok(v) => v,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Failed to encode command"}"#))
                .unwrap();
        }
    };
    cmd.push(b'\n');

    let agent_connected = *state.agent_connected.read().unwrap();
    let command_tx = state.agent_command_tx.lock().unwrap().clone();

    if !agent_connected {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Agent not connected"}"#))
            .unwrap();
    }

    let sent = if let Some(handle) = command_tx {
        handle
            .tx
            .send(String::from_utf8_lossy(&cmd).into_owned())
            .is_ok()
    } else {
        false
    };

    if !sent {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Agent not connected"}"#))
            .unwrap();
    }

    {
        let headers = headers.clone();
        let target = format!("{provider}:{id}");
        with_db(state.db.clone(), move |db| {
            record_audit_blocking(
                db,
                &headers,
                &admin_user,
                "container_action",
                &target,
                &action,
            );
        })
        .await;
    }

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(12) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let result = {
            let mut results = state.action_results.write().unwrap();
            results.remove(&request_id)
        };
        if let Some(result) = result {
            if result.success {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"success":true}"#))
                    .unwrap();
            }
            let error = result.error.unwrap_or_else(|| "Action failed".to_string());
            let body = serde_json::json!({ "error": error });
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap();
        }
    }

    state.action_results.write().unwrap().remove(&request_id);

    Response::builder()
        .status(StatusCode::GATEWAY_TIMEOUT)
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"error":"Timed out waiting for agent to confirm action"}"#,
        ))
        .unwrap()
}

pub async fn serve_upload_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    if get_session(&headers, &state.sessions).is_none() {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Forbidden"))
            .unwrap();
    }
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap();
    }
    let path = format!("data/uploads/{}", filename);
    match fs::read(&path) {
        Ok(bytes) => {
            let content_type = match filename
                .rsplit('.')
                .next()
                .unwrap_or("")
                .to_lowercase()
                .as_str()
            {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "ico" => "image/x-icon",
                "svg" => "image/svg+xml",
                _ => "application/octet-stream",
            };
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .body(Body::from(bytes))
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    }
}

pub async fn integration_data_handler(
    headers: HeaderMap,
    Path(id): Path<i64>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "integration_data", 30, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let is_admin = session.as_ref().map(|s| s.role == "Admin").unwrap_or(false);

    let app = with_db(state.db.clone(), move |db| {
        let mut apps = crate::db::load_apps_from_db(db);
        apps.retain(|a| a.id == id);
        apps.pop()
    })
    .await;

    match &app {
        Some(app) => {
            if app.integration_type.as_str() != "rss" && !is_admin {
                return Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"error":"Forbidden"}"#))
                    .unwrap();
            }
        }
        None => {
            if !is_admin {
                return Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"error":"Forbidden"}"#))
                    .unwrap();
            } else {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("{}"))
                    .unwrap();
            }
        }
    }

    let app = app.unwrap();

    let accept_invalid = {
        let cache = state.settings_cache.read().unwrap();
        cache
            .get("accept_invalid_certs")
            .map(|s| s == "1")
            .unwrap_or(false)
    };

    if let Some(data) = crate::integrations::fetch_integration_data(&app, accept_invalid).await {
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(data.to_string()))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("{}"))
        .unwrap()
}

pub async fn integration_action_handler(
    headers: HeaderMap,
    Path(id): Path<i64>,
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }
    if !validate_csrf(&headers, &state.sessions, None) {
        return csrf_forbidden_response();
    }

    let accept_invalid = {
        let cache = state.settings_cache.read().unwrap();
        cache
            .get("accept_invalid_certs")
            .map(|s| s == "1")
            .unwrap_or(false)
    };

    let action = payload.get("action").cloned().unwrap_or_default();
    let app = with_db(state.db.clone(), move |db| {
        let mut apps = crate::db::load_apps_from_db(db);
        apps.retain(|a| a.id == id);
        apps.pop()
    })
    .await;

    if let Some(app) = app {
        if let Some(data) =
            crate::integrations::execute_integration_action(&app, &action, accept_invalid).await
        {
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(data.to_string()))
                .unwrap();
        }
    }

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("{}"))
        .unwrap()
}

pub async fn wake_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Unauthorized"}"#))
            .unwrap();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let mac_str = with_db(state.db.clone(), move |db| {
                fetch_wol_device_mac_address(db, id)
            })
            .await;
            if let Some(mac_str) = mac_str {
                if let Some(mac_bytes) = parse_mac(&mac_str) {
                    let mut magic_packet = vec![0xFF; 6];
                    for _ in 0..16 {
                        magic_packet.extend(&mac_bytes);
                    }
                    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
                        let _ = socket.set_broadcast(true);
                        let _ = socket.send_to(&magic_packet, "255.255.255.255:9");
                        return Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"success":true}"#))
                            .unwrap();
                    }
                }
            }
        }
    }
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"error":"Failed to send magic packet"}"#))
        .unwrap()
}

pub async fn list_wol_devices_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }

    let devices = with_db(state.db.clone(), load_wol_devices_from_db).await;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&devices).unwrap_or_else(|_| "[]".to_string()),
        ))
        .unwrap()
}

pub async fn add_wol_device_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    let name = form.get("name").cloned().unwrap_or_default();
    let mac_address = form.get("mac_address").cloned().unwrap_or_default();
    let ip_address = form.get("ip_address").cloned().unwrap_or_default();
    let icon = form.get("icon").cloned().unwrap_or_default();

    if !name.is_empty() && !mac_address.is_empty() {
        let admin_user = session
            .as_ref()
            .map(|s| s.username.clone())
            .unwrap_or_default();
        let headers = headers.clone();
        with_db(state.db.clone(), move |db| {
            if insert_wol_device(db, &name, &mac_address, &ip_address, &icon).is_ok() {
                record_audit_blocking(
                    db,
                    &headers,
                    &admin_user,
                    "wol_device_create",
                    &name,
                    &mac_address,
                );
            }
        })
        .await;
    }
    Redirect::to("/admin/settings").into_response()
}

pub async fn delete_wol_device_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let admin_user = session
                .as_ref()
                .map(|s| s.username.clone())
                .unwrap_or_default();
            let headers = headers.clone();
            let audit_id_str = id_str.clone();
            with_db(state.db.clone(), move |db| {
                if delete_wol_device(db, id).is_ok() {
                    record_audit_blocking(
                        db,
                        &headers,
                        &admin_user,
                        "wol_device_delete",
                        &audit_id_str,
                        "",
                    );
                }
            })
            .await;
        }
    }
    Redirect::to("/admin/settings").into_response()
}

pub async fn reorder_apps_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }

    #[derive(Deserialize)]
    struct ReorderPayload {
        ids: Vec<i64>,
        csrf_token: String,
    }

    let Ok(payload) = serde_json::from_str::<ReorderPayload>(&body) else {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Invalid payload"}),
        )
        .into_response();
    };

    let mut csrf_form = HashMap::new();
    csrf_form.insert("csrf_token".to_string(), payload.csrf_token);

    if !validate_csrf(&headers, &state.sessions, Some(&csrf_form)) {
        return csrf_forbidden_response().into_response();
    }

    let result = with_db(state.db.clone(), move |db| {
        update_app_order(db, &payload.ids)
    })
    .await;

    match result {
        Ok(()) => api_json(StatusCode::OK, serde_json::json!({"success": true})).into_response(),
        Err(error) => api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"success": false, "error": error}),
        )
        .into_response(),
    }
}

fn rss_feed_api_key_valid(integration_type: &str, api_key: &str) -> bool {
    if integration_type != "rss" {
        return true;
    }
    let trimmed = api_key.trim();
    if trimmed.is_empty() || trimmed == "Configured — leave blank to keep unchanged" {
        return true;
    }
    sanitize_rss_feed_url(trimmed).is_some()
}

fn encrypt_integration_api_key(integration_type: &str, api_key: &str) -> String {
    if api_key.trim().is_empty() {
        return api_key.to_string();
    }
    let value_to_store = if integration_type == "rss" {
        sanitize_rss_feed_url(api_key).unwrap_or_else(|| api_key.trim().to_string())
    } else {
        api_key.to_string()
    };
    crate::secrets::encrypt_value(&value_to_store).unwrap_or(value_to_store)
}
