//! Activity viewport and presence API handlers.

use super::imports::*;
use crate::activity::{is_deep_idle, signal_gui_session_end, signal_gui_session_start, signal_viewport, MAX_VISIBLE_APPS};
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ViewportBody {
    pub ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct PresenceBody {
    #[serde(default)]
    pub active: bool,
}

pub async fn activity_viewport_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ViewportBody>,
) -> impl IntoResponse {
    if get_session(&headers, &state.sessions).is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "unauthorized" })),
        )
            .into_response();
    }
    if is_deep_idle(&state) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "server_idle" })),
        )
            .into_response();
    }
    let ids: Vec<i64> = body.ids.into_iter().take(MAX_VISIBLE_APPS).collect();
    signal_viewport(&state, ids);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true })),
    )
        .into_response()
}

pub async fn activity_presence_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PresenceBody>,
) -> impl IntoResponse {
    if get_session(&headers, &state.sessions).is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "unauthorized" })),
        )
            .into_response();
    }
    if body.active {
        signal_gui_session_start(&state);
    } else {
        signal_gui_session_end(&state);
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true })),
    )
        .into_response()
}
