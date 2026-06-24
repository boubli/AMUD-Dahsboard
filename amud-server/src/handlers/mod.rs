mod admin;
mod api_tokens;
mod apps;
mod auth;
mod backup;
mod categories;
mod dashboard;
mod discover;
mod embed;
mod feed_categories;
mod imports;
mod oidc;
mod pages;
mod share;
mod status;
mod system;
mod users;
mod webhook_routes;
mod widgets;

pub use admin::*;
pub use api_tokens::*;
pub use apps::*;
pub use auth::*;
pub use backup::*;
pub use categories::*;
pub use dashboard::*;
pub use discover::*;
pub use embed::*;
pub use feed_categories::*;
pub use oidc::*;
pub use pages::*;
pub use share::*;
pub use status::*;
pub use system::*;
pub use users::*;
pub use webhook_routes::*;
pub use widgets::*;

use crate::auth::rate_limit_response;
use crate::models::AppState;
use crate::security::{client_ip, enforce_rate_limit, RateLimitConfig};
use axum::body::Body;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;
use std::time::Duration;

pub(crate) fn apply_csp_nonce(html: String, nonce: &str) -> String {
    html.replace("{{csp_nonce}}", nonce)
}

pub(crate) fn check_api_rate_limit(
    state: &AppState,
    headers: &HeaderMap,
    bucket: &str,
    max: usize,
    window_secs: u64,
) -> Option<Response> {
    let key = format!("{}:{}", bucket, client_ip(headers));
    if !enforce_rate_limit(
        &state.api_rate_limits,
        &key,
        RateLimitConfig {
            max,
            window: Duration::from_secs(window_secs),
        },
    ) {
        Some(rate_limit_response())
    } else {
        None
    }
}

pub(crate) fn api_json(status: StatusCode, value: serde_json::Value) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .unwrap()
}
