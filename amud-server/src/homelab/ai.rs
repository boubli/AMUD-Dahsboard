//! AI / LLM integrations (Ollama, Open WebUI).

use super::{auth_header, get_json};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;

pub async fn fetch_ollama(client: &Client, base_url: &str, _api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let tags = get_json(client, &format!("{base}/api/tags"), None).await?;
    let models = tags
        .get("models")
        .and_then(|m| m.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(0);
    let running = get_json(client, &format!("{base}/api/ps"), None)
        .await
        .and_then(|ps| {
            ps.get("models")
                .and_then(|m| m.as_array())
                .or_else(|| ps.as_array())
                .map(|a| a.len() as u64)
        })
        .unwrap_or(0);
    Some(json!({
        "type": "ollama",
        "tier2": true,
        "models": models,
        "running": running,
        "status": "Online",
    }))
}

pub async fn fetch_open_webui(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if !key.is_empty() && key != "none" {
        let auth = auth_header(key);
        if let Some(body) = get_json(client, &format!("{base}/api/models"), Some(&auth)).await {
            let models = body
                .get("data")
                .and_then(|d| d.as_array())
                .or_else(|| body.as_array())
                .map(|a| a.len() as u64)
                .unwrap_or(0);
            return Some(json!({
                "type": "open_webui",
                "tier2": true,
                "models": models,
                "status": "Online",
            }));
        }
    }
    let started = Instant::now();
    let url = format!("{base}/health");
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let latency_ms = started.elapsed().as_millis() as u64;
    Some(json!({
        "type": "open_webui",
        "tier2": true,
        "status": "Online",
        "latency_ms": latency_ms,
    }))
}
