use super::imports::*;

pub async fn list_boards_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = match get_session(&headers, &state.sessions) {
        Some(s) => s,
        None => {
            return api_json(
                StatusCode::UNAUTHORIZED,
                serde_json::json!({ "error": "Unauthorized" }),
            );
        }
    };
    let boards = with_db(state.db.clone(), move |db| {
        crate::boards::list_dashboards(db, &session.username)
    })
    .await;
    api_json(StatusCode::OK, serde_json::json!({ "boards": boards }))
}

pub async fn create_board_handler(
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
    let name = form.get("name").cloned().unwrap_or_else(|| "Board".into());
    let owner = form.get("owner").cloned().unwrap_or_else(|| "admin".into());
    let id = with_db(state.db.clone(), move |db| {
        crate::boards::create_dashboard(db, &name, &owner)
    })
    .await;
    Redirect::to(&format!("/admin/settings?tab=boards&created={id}")).into_response()
}
