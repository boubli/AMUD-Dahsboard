use super::imports::*;

// User Management Handlers
pub async fn list_users_handler(
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

    let users = with_db(state.db.clone(), load_users_json).await;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&users).unwrap(),
        ))
        .unwrap()
}

#[derive(Deserialize)]
pub struct AddUserForm {
    username: String,
    password: Option<String>,
    role: String,
}

pub async fn add_user_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<AddUserForm>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "user_mgmt", 10, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return api_json(
                StatusCode::FORBIDDEN,
                serde_json::json!({"error": "Forbidden"}),
            )
        }
    };
    if !validate_csrf(&headers, &state.sessions, None) {
        return api_json(
            StatusCode::FORBIDDEN,
            serde_json::json!({"error": "Forbidden"}),
        );
    }

    if !valid_user_role(&form.role) {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Role must be Admin or Guest."}),
        );
    }

    let pass = form.password.unwrap_or_default();
    if pass.is_empty() {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Password is required for new users."}),
        );
    }
    let p_hash = hash_password(&pass);
    let username = form.username.trim().to_string();
    let role = form.role.clone();
    let headers = headers.clone();
    let ok = with_db(state.db.clone(), move |db| {
        match db.execute(
            "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
            params![username, p_hash, role],
        ) {
            Ok(_) => {
                record_audit_blocking(db, &headers, &admin_user, "user_create", &username, &role);
                true
            }
            Err(_) => false,
        }
    })
    .await;

    if ok {
        api_json(StatusCode::OK, serde_json::json!({"success": true}))
    } else {
        api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Username already exists or invalid."}),
        )
    }
}

#[derive(Deserialize)]
pub struct EditUserForm {
    id: i64,
    username: String,
    password: Option<String>,
    role: String,
}

pub async fn edit_user_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<EditUserForm>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "user_mgmt", 10, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return api_json(
                StatusCode::FORBIDDEN,
                serde_json::json!({"error": "Forbidden"}),
            )
        }
    };
    if !validate_csrf(&headers, &state.sessions, None) {
        return api_json(
            StatusCode::FORBIDDEN,
            serde_json::json!({"error": "Forbidden"}),
        );
    }

    if !valid_user_role(&form.role) {
        return api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Role must be Admin or Guest."}),
        );
    }

    let username = form.username.trim().to_string();
    let role = form.role.clone();
    let id = form.id;
    let headers = headers.clone();
    let password = form.password.filter(|p| !p.trim().is_empty());
    let details = with_db(state.db.clone(), move |db| {
        let result = if let Some(pass) = password {
            let p_hash = hash_password(&pass);
            db.execute(
                "UPDATE users SET username = ?, password_hash = ?, role = ? WHERE id = ?",
                params![username, p_hash, role, id],
            )
            .map(|_| "password and profile updated")
            .map_err(|_| "Update failed.")
        } else {
            db.execute(
                "UPDATE users SET username = ?, role = ? WHERE id = ?",
                params![username, role, id],
            )
            .map(|_| "profile updated")
            .map_err(|_| "Update failed.")
        };
        if let Ok(detail) = &result {
            record_audit_blocking(db, &headers, &admin_user, "user_update", &username, detail);
        }
        result
    })
    .await;

    if details.is_ok() {
        api_json(StatusCode::OK, serde_json::json!({"success": true}))
    } else {
        api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Update failed."}),
        )
    }
}

#[derive(Deserialize)]
pub struct DeleteUserForm {
    id: i64,
}

pub async fn delete_user_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<DeleteUserForm>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "user_mgmt", 10, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return api_json(
                StatusCode::FORBIDDEN,
                serde_json::json!({"error": "Forbidden"}),
            )
        }
    };
    if !validate_csrf(&headers, &state.sessions, None) {
        return api_json(
            StatusCode::FORBIDDEN,
            serde_json::json!({"error": "Forbidden"}),
        );
    }

    let self_username = session.as_ref().map(|s| s.username.clone());
    let headers = headers.clone();
    let id = form.id;
    let result = with_db(state.db.clone(), move |db| {
        delete_user_by_id(db, id, self_username.as_deref(), &admin_user, &headers)
    })
    .await;

    match result {
        Ok(()) => api_json(StatusCode::OK, serde_json::json!({"success": true})),
        Err(DeleteUserError::NotFound) => api_json(
            StatusCode::NOT_FOUND,
            serde_json::json!({"error": "User not found."}),
        ),
        Err(DeleteUserError::SelfDelete) => api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "You cannot delete your own account."}),
        ),
        Err(DeleteUserError::LastAdmin) => api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Cannot delete the last Admin account."}),
        ),
        Err(DeleteUserError::Failed) => api_json(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Delete failed."}),
        ),
    }
}
