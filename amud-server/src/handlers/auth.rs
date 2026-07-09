use super::imports::*;

pub async fn login_page(
    Extension(csp): Extension<CspNonce>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let settings = state.settings_cache.read().unwrap().clone();
    let branding = branding_from_settings(&settings);
    let custom_css = settings.get("custom_css").map(|s| s.as_str()).unwrap_or("");
    let active_theme_id = settings
        .get("active_theme_id")
        .map(|s| s.as_str())
        .unwrap_or("default");

    let login_tmpl = include_str!("../../../ui/templates/login.html");
    let result = apply_shared_branding(
        login_tmpl.to_string(),
        &BrandingRenderOptions {
            branding: &branding,
            custom_css,
            default_tagline: "Access administrative operations cockpit",
            active_theme_id,
        },
    );
    let theme_config = build_theme_scheduler_json(&settings, &branding.theme_mode);
    let oidc_enabled = settings
        .get("oidc_enabled")
        .map(|s| s.as_str())
        .unwrap_or("0")
        == "1";
    let oidc_block = if oidc_enabled {
        r#"<a href="/auth/oidc/login" class="btn btn-secondary" style="width:100%; text-align:center; text-decoration:none; display:block; padding:0.75rem;">Sign in with SSO</a>"#.to_string()
    } else {
        String::new()
    };
    let result = result.replace("{{theme_scheduler_config}}", &theme_config);
    let result = result.replace("{{oidc_sso_block}}", &oidc_block);
    Html(apply_csp_nonce(result, &csp.0))
}

pub async fn login_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "login_ip", 30, 300) {
        return resp;
    }

    let username = form
        .get("username")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let password = form.get("password").cloned().unwrap_or_default();

    if login_rate_limited(&state.login_attempts, &username) {
        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(axum::body::Body::from(
                "Too many failed login attempts. Try again later.",
            ))
            .unwrap();
    }

    let username_db = username.clone();
    let password_db = password.clone();

    let ldap_cfg = {
        let cache = state.settings_cache.read().unwrap();
        crate::ldap_auth::ldap_settings_from_map(&cache)
    };
    let mut ldap_ok = false;
    if ldap_cfg.enabled && !password.is_empty() {
        ldap_ok = crate::ldap_auth::ldap_authenticate(&ldap_cfg, &username, &password)
            .await
            .is_ok();
    }

    let login = if ldap_ok {
        crate::db::LoginDbResult {
            success: true,
            role: "Guest".to_string(),
            must_change_password: false,
        }
    } else {
        with_db(state.db.clone(), move |db| {
            process_login(db, &username_db, &password_db)
        })
        .await
    };

    if login.success {
        clear_failed_logins(&state.login_attempts, &username);
        let headers = headers.clone();
        let username_audit = username.clone();
        let role_audit = login.role.clone();
        with_db(state.db.clone(), move |db| {
            record_audit_blocking(
                db,
                &headers,
                &username_audit,
                "login",
                &username_audit,
                &role_audit,
            );
        })
        .await;
        let role = login.role;
        let token = generate_session_token();
        let csrf_token = generate_session_token();

        state.sessions.write().unwrap().insert(
            token.clone(),
            Session {
                username: username.clone(),
                role: role.clone(),
                expires_at_epoch: now_epoch_secs() + 86_400,
                csrf_token,
            },
        );

        let cookie = session_cookie(&token);
        let redirect_to = if login.must_change_password && role == "Admin" {
            "/admin/settings"
        } else {
            "/"
        };

        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::SET_COOKIE, cookie)
            .header(header::LOCATION, redirect_to)
            .body(axum::body::Body::empty())
            .unwrap()
    } else {
        // Keep missing-user and wrong-password timing closer by doing an Argon2id hash
        // even when no stored hash exists.
        let _ = hash_password(&password);
        record_failed_login(&state.login_attempts, &username);
        Redirect::to("/login?error=1").into_response()
    }
}

pub async fn logout_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    form: Option<Form<HashMap<String, String>>>,
) -> impl IntoResponse {
    if let Some(Form(form)) = &form {
        if !validate_csrf(&headers, &state.sessions, Some(form)) {
            return csrf_forbidden_response();
        }
    }
    let session = get_session(&headers, &state.sessions);
    let username = session
        .as_ref()
        .map(|s| s.username.as_str())
        .unwrap_or("unknown");
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
    if username != "unknown" {
        let headers = headers.clone();
        let username = username.to_string();
        with_db(state.db.clone(), move |db| {
            record_audit_blocking(db, &headers, &username, "logout", &username, "");
        })
        .await;
    }

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::SET_COOKIE, expired_session_cookie())
        .header(header::LOCATION, "/")
        .body(axum::body::Body::empty())
        .unwrap()
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    let limited_telemetry = match &session {
        None => true,
        Some(s) => s.role == "Guest",
    };
    let public = if limited_telemetry {
        let settings = state.settings_cache.read().unwrap();
        telemetry_public_from_cache(&settings)
    } else {
        false
    };
    ws.on_upgrade(move |socket| handle_ws_session(socket, state, limited_telemetry, public))
}

async fn handle_ws_session(
    mut socket: WebSocket,
    state: Arc<AppState>,
    limited_telemetry: bool,
    _public_at_connect: bool,
) {
    let mut rx = state.telemetry_broadcast.subscribe();

    let public = if limited_telemetry {
        let settings = state.settings_cache.read().unwrap();
        telemetry_public_from_cache(&settings)
    } else {
        false
    };
    let live_bundle = crate::telemetry_broadcast::WsTelemetryBundle::from_state(&state);
    let first_frame =
        crate::telemetry_broadcast::ws_frame_from_bundle(&live_bundle, limited_telemetry, public);
    if socket
        .send(WsMessage::Text(first_frame.to_string()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        if rx.changed().await.is_err() {
            break;
        }
        let bundle = rx.borrow().clone();
        let public = if limited_telemetry {
            let settings = state.settings_cache.read().unwrap();
            telemetry_public_from_cache(&settings)
        } else {
            false
        };
        let frame =
            crate::telemetry_broadcast::ws_frame_from_bundle(&bundle, limited_telemetry, public);

        if socket
            .send(WsMessage::Text(frame.to_string()))
            .await
            .is_err()
        {
            break;
        }
    }
}
