use super::imports::*;

pub async fn homepage_import_preview_handler(
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
    let services = form.get("services_yaml").cloned().unwrap_or_default();
    let widgets = form.get("widgets_yaml").cloned();
    match crate::homepage_import::import_preview_json(
        &services,
        widgets.as_deref().filter(|s| !s.trim().is_empty()),
    ) {
        Ok(preview) => api_json(StatusCode::OK, preview),
        Err(e) => api_json(StatusCode::BAD_REQUEST, serde_json::json!({ "error": e })),
    }
}

pub async fn homepage_import_apply_handler(
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
    let services = form.get("services_yaml").cloned().unwrap_or_default();
    let apps = match crate::homepage_import::parse_homepage_services_yaml(&services) {
        Ok(a) => a,
        Err(e) => {
            return api_json(StatusCode::BAD_REQUEST, serde_json::json!({ "error": e }));
        }
    };
    let imported = with_db(state.db.clone(), move |db| {
        let mut count = 0usize;
        for app in apps {
            let exists: i64 = db
                .query_row(
                    "SELECT COUNT(*) FROM apps WHERE lower(name) = lower(?) OR url = ?",
                    params![app.name, app.url],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if exists > 0 {
                continue;
            }
            let category = crate::db::resolve_app_category(db, &app.category);
            let sort_order = crate::db::next_app_sort_order(db);
            let _ = db.execute(
                "INSERT INTO apps (name, url, icon, description, category, node_tag, sort_order, guest_visible, embed_mode, integration_type, api_key) VALUES (?, ?, ?, ?, ?, 'Local', ?, 1, 'link', ?, ?)",
                params![
                    app.name,
                    app.url,
                    app.icon,
                    app.description,
                    category,
                    sort_order,
                    app.integration_type,
                    app.api_key,
                ],
            );
            count += 1;
        }
        count
    })
    .await;
    {
        let db = state.db.lock().unwrap();
        record_audit_blocking(
            &db,
            &headers,
            "Admin",
            "import",
            "homepage",
            &format!("Imported {imported} apps from Homepage YAML"),
        );
    }
    Redirect::to("/admin/settings?tab=integrations&imported=homepage").into_response()
}

pub async fn homarr_import_apply_handler(
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
    let payload = form.get("homarr_json").cloned().unwrap_or_default();
    let apps = match crate::homarr_import::parse_homarr_export(&payload) {
        Ok(a) => a,
        Err(e) => {
            return api_json(StatusCode::BAD_REQUEST, serde_json::json!({ "error": e }));
        }
    };
    let imported = with_db(state.db.clone(), move |db| {
        let mut count = 0usize;
        for app in apps {
            let exists: i64 = db
                .query_row(
                    "SELECT COUNT(*) FROM apps WHERE lower(name) = lower(?) OR url = ?",
                    params![app.name, app.url],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if exists > 0 {
                continue;
            }
            let category = crate::db::resolve_app_category(db, &app.category);
            let sort_order = crate::db::next_app_sort_order(db);
            let _ = db.execute(
                "INSERT INTO apps (name, url, icon, description, category, node_tag, sort_order, guest_visible, embed_mode, integration_type, api_key) VALUES (?, ?, ?, '', ?, 'Local', ?, 1, 'link', ?, ?)",
                params![
                    app.name,
                    app.url,
                    app.icon,
                    category,
                    sort_order,
                    app.integration_type,
                    app.api_key,
                ],
            );
            count += 1;
        }
        count
    })
    .await;
    {
        let db = state.db.lock().unwrap();
        record_audit_blocking(
            &db,
            &headers,
            "Admin",
            "import",
            "homarr",
            &format!("Imported {imported} apps from Homarr export"),
        );
    }
    Redirect::to("/admin/settings?tab=integrations&imported=homarr").into_response()
}

pub async fn integration_manifest_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if get_session(&headers, &state.sessions).is_none() {
        return api_json(
            StatusCode::UNAUTHORIZED,
            serde_json::json!({ "error": "Unauthorized" }),
        );
    }
    api_json(
        StatusCode::OK,
        crate::integration_registry::integration_manifest_json(),
    )
}
