use super::imports::*;

// Webhook API handlers
pub async fn list_webhooks_handler(
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

    let list = with_db(state.db.clone(), load_webhooks_json).await;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&list).unwrap(),
        ))
        .unwrap()
}

pub async fn add_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "webhook", 20, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let name = form
        .get("name")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let url = form
        .get("url")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let event_types_raw = form
        .get("event_types")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let is_active = form
        .get("is_active")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    if name.is_empty() || url.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Name and URL are required"}"#,
            ))
            .unwrap();
    }

    let event_types = match normalize_webhook_event_types(&event_types_raw) {
        Some(v) => v,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"Invalid or empty event_types"}"#,
                ))
                .unwrap();
        }
    };

    if !url_allowed_for_webhook(
        &url,
        state
            .settings_cache
            .read()
            .unwrap()
            .get("webhooks_allow_private_ips")
            .map(|s| s == "1")
            .unwrap_or(false),
    ) {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Webhook URL is not allowed (blocked private/loopback targets)"}"#,
            ))
            .unwrap();
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"URL must start with http:// or https://"}"#,
            ))
            .unwrap();
    }

    let headers = headers.clone();
    let result = with_db(state.db.clone(), move |db| {
        db.execute(
            "INSERT INTO webhooks (name, url, event_types, is_active) VALUES (?, ?, ?, ?)",
            params![name, url, event_types, is_active],
        )
        .map(|_| {
            record_audit_blocking(
                db,
                &headers,
                &admin_user,
                "webhook_create",
                &name,
                "webhook added",
            );
        })
    })
    .await;

    match result {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(format!(
                r#"{{"error":"Database error: {e}"}}"#
            )))
            .unwrap(),
    }
}

pub async fn edit_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "webhook", 20, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    let id = match id_str.parse::<i64>() {
        Ok(v) => v,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Invalid Webhook ID"}"#))
                .unwrap()
        }
    };

    let name = form
        .get("name")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let url_input = form
        .get("url")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let event_types_raw = form
        .get("event_types")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let is_active = form
        .get("is_active")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    if name.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Name is required"}"#))
            .unwrap();
    }

    let event_types = match normalize_webhook_event_types(&event_types_raw) {
        Some(v) => v,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"Invalid or empty event_types"}"#,
                ))
                .unwrap();
        }
    };

    if !url_input.is_empty() {
        if !url_input.starts_with("http://") && !url_input.starts_with("https://") {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"URL must start with http:// or https://"}"#,
                ))
                .unwrap();
        }
        if !url_allowed_for_webhook(
            &url_input,
            state
                .settings_cache
                .read()
                .unwrap()
                .get("webhooks_allow_private_ips")
                .map(|s| s == "1")
                .unwrap_or(false),
        ) {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"Webhook URL is not allowed (blocked private/loopback targets)"}"#,
                ))
                .unwrap();
        }
    }

    let headers = headers.clone();
    let result = with_db(state.db.clone(), move |db| {
        let url = if url_input.is_empty() {
            db.query_row(
                "SELECT url FROM webhooks WHERE id = ?",
                params![id],
                |row| row.get::<_, String>(0),
            )
            .map_err(|_| "not_found")
        } else {
            Ok(url_input)
        }?;
        db.execute(
            "UPDATE webhooks SET name = ?, url = ?, event_types = ?, is_active = ? WHERE id = ?",
            params![name, url, event_types, is_active, id],
        )
        .map(|_| {
            record_audit_blocking(
                db,
                &headers,
                &admin_user,
                "webhook_update",
                &format!("id:{id}"),
                &name,
            );
        })
        .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap(),
        Err(err) if err == "not_found" => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Webhook not found"}"#))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(format!(
                r#"{{"error":"Database error: {e}"}}"#
            )))
            .unwrap(),
    }
}

pub async fn delete_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "webhook", 20, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    if let Ok(id) = id_str.parse::<i64>() {
        let headers = headers.clone();
        with_db(state.db.clone(), move |db| {
            db.execute("DELETE FROM webhooks WHERE id = ?", params![id])
                .ok();
            record_audit_blocking(
                db,
                &headers,
                &admin_user,
                "webhook_delete",
                &format!("id:{id}"),
                "webhook removed",
            );
        })
        .await;
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

pub async fn test_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "webhook_test", 5, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    let id = match id_str.parse::<i64>() {
        Ok(v) => v,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Invalid ID"}"#))
                .unwrap()
        }
    };

    let headers = headers.clone();
    let webhook = with_db(state.db.clone(), move |db| fetch_webhook_by_id(db, id)).await;

    if let Some((name, url)) = webhook {
        let audit_name = name.clone();
        with_db(state.db.clone(), move |db| {
            record_audit_blocking(
                db,
                &headers,
                &admin_user,
                "webhook_test",
                &format!("id:{id}"),
                &audit_name,
            );
        })
        .await;
        let accept_invalid = {
            let cache = state.settings_cache.read().unwrap();
            cache
                .get("accept_invalid_certs")
                .map(|s| s == "1")
                .unwrap_or(false)
        };
        let allow_private = {
            let cache = state.settings_cache.read().unwrap();
            cache
                .get("webhooks_allow_private_ips")
                .map(|s| s == "1")
                .unwrap_or(false)
        };
        let delivered = send_webhook_notification(
            crate::http_client::select_http_client(&state.http_clients, accept_invalid),
            url,
            name,
            "test",
            "Test Container",
            999,
            "running",
            "Docker",
            allow_private,
        )
        .await;

        if delivered {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"success":true}"#))
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"Webhook delivery failed. Check URL and destination."}"#,
                ))
                .unwrap()
        }
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Webhook not found"}"#))
            .unwrap()
    }
}
