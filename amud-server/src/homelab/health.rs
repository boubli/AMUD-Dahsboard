//! Tier 3 health-check-only integrations.

use super::{auth_header, get_json};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;

pub const HEALTH_ONLY_TYPES: &[&str] = &[
    "kodi",
    "drone",
    "hubitat",
    "smartthings",
    "iobroker",
    "blue_iris",
    "shinobi",
    "agent_dvr",
    "wireguard_ui",
    "openvpn",
    "seaweedfs",
    "garage",
];

pub fn is_health_only(integration_type: &str) -> bool {
    HEALTH_ONLY_TYPES.contains(&integration_type)
}

fn probe_path(integration_type: &str) -> &'static str {
    match integration_type {
        "gitea" | "forgejo" => "/api/v1/version",
        "gitlab" => "/api/v4/version",
        "jenkins" => "/api/json",
        "drone" => "/api/user",
        "minio" => "/minio/health/live",
        "headscale" => "/health",
        "kopia" => "/api/v1/repo/status",
        "restic" => "/",
        "duplicati" => "/api/v1/status",
        "urbackup" => "/status",
        "kodi" => "/jsonrpc",
        "stash" => "/graphql",
        "calibre_web" => "/opds",
        "iobroker" => "/",
        "shinobi" => "/",
        "garage" => "/health",
        "seaweedfs" => "/cluster/status",
        _ => "/",
    }
}

fn extract_version(integration_type: &str, body: &Value) -> String {
    match integration_type {
        "gitea" | "forgejo" => body.get("version").and_then(|v| v.as_str()).unwrap_or("—"),
        "gitlab" => body.get("version").and_then(|v| v.as_str()).unwrap_or("—"),
        "jenkins" => "—",
        "minio" => "—",
        _ => body.get("version").and_then(|v| v.as_str()).unwrap_or("—"),
    }
    .to_string()
}

pub async fn fetch_health_integration(
    client: &Client,
    integration_type: &str,
    base_url: &str,
    api_key: &str,
) -> Option<Value> {
    if !is_health_only(integration_type) {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    let path = probe_path(integration_type);
    let url = if path == "/" {
        base.to_string()
    } else {
        format!("{base}{path}")
    };
    let auth = if api_key.trim().is_empty() || api_key.trim() == "none" {
        None
    } else if api_key.contains('|') {
        if let Some((user, pass)) = super::parse_pipe_credential(api_key) {
            Some(format!("Basic {}", super::basic_auth(user, pass)))
        } else {
            None
        }
    } else {
        Some(auth_header(api_key.trim()))
    };
    let started = Instant::now();
    if let Some(body) = get_json(client, &url, auth.as_deref()).await {
        let latency_ms = started.elapsed().as_millis() as u64;
        return Some(json!({
            "type": integration_type,
            "health_only": true,
            "status": "Online",
            "version": extract_version(integration_type, &body),
            "latency_ms": latency_ms,
        }));
    }
    // Fallback: root URL HEAD/GET
    let started = Instant::now();
    let mut req = client.get(base);
    if let Some(a) = auth.as_deref() {
        req = req.header("Authorization", a);
    }
    let resp = req.send().await.ok()?;
    if resp.status().is_success() || resp.status().is_redirection() {
        Some(json!({
            "type": integration_type,
            "health_only": true,
            "status": "Online",
            "version": "—",
            "latency_ms": started.elapsed().as_millis() as u64,
        }))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_only_types_exclude_promoted_full_cards() {
        assert!(is_health_only("kodi"));
        assert!(!is_health_only("gitea"));
        assert!(!is_health_only("grafana"));
    }
}
