//! Shared reqwest clients for background pollers (avoids per-tick TLS allocator churn).

use std::time::Duration;

pub struct SharedHttpClients {
    pub strict: reqwest::Client,
    pub permissive: reqwest::Client,
    pub homelab: reqwest::Client,
}

pub fn build_shared_http_clients() -> SharedHttpClients {
    let strict = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let permissive = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let homelab = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    SharedHttpClients {
        strict,
        permissive,
        homelab,
    }
}

pub fn select_http_client(
    clients: &SharedHttpClients,
    accept_invalid: bool,
) -> &reqwest::Client {
    if accept_invalid {
        &clients.permissive
    } else {
        &clients.strict
    }
}
