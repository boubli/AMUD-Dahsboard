use super::imports::*;

pub async fn list_categories_handler(
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

    let categories = with_db(state.db.clone(), load_categories_json).await;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&categories).unwrap(),
        ))
        .unwrap()
}

pub async fn add_category_handler(
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
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let name = form.get("name").cloned().unwrap_or_default();
    let color = form
        .get("color")
        .cloned()
        .unwrap_or_else(|| "#64748b".to_string());
    let sort_order = form
        .get("sort_order")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);

    if name.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Category name is required"}"#,
            ))
            .unwrap();
    }

    let ok = with_db(state.db.clone(), move |db| {
        db.execute(
            "INSERT INTO categories (name, color, sort_order) VALUES (?, ?, ?)",
            params![name, color, sort_order],
        )
        .is_ok()
    })
    .await;

    if ok {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::CONFLICT)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Category already exists"}"#,
            ))
            .unwrap()
    }
}

pub async fn delete_category_handler(
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
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let result = with_db(state.db.clone(), move |db| delete_category_by_id(db, id)).await;
            return match result {
                Err(CategoryDeleteError::LastCategory) => api_json(
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({"error": "Cannot delete the last category."}),
                ),
                Err(CategoryDeleteError::NotFound) => api_json(
                    StatusCode::NOT_FOUND,
                    serde_json::json!({"error": "Category not found."}),
                ),
                Err(CategoryDeleteError::DeleteFailed) => api_json(
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({"error": "Failed to delete category."}),
                ),
                Ok(()) => api_json(StatusCode::OK, serde_json::json!({"success": true})),
            };
        }
    }

    api_json(StatusCode::OK, serde_json::json!({"success": true}))
}

pub async fn edit_category_handler(
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
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let name = form.get("name").cloned().unwrap_or_default();
            let color = form
                .get("color")
                .cloned()
                .unwrap_or_else(|| "#64748b".to_string());
            let sort_order = form
                .get("sort_order")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0);
            if !name.is_empty() {
                with_db(state.db.clone(), move |db| {
                    update_category_by_id(db, id, &name, &color, sort_order);
                })
                .await;
            }
        }
    }

    api_json(StatusCode::OK, serde_json::json!({"success": true}))
}
