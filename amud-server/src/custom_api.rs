//! User-defined HTTP integration (Homepage Custom API parity).

use crate::security::url_allowed_for_health_check;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct CustomApiConfig {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub fields: std::collections::BTreeMap<String, String>,
}

fn default_method() -> String {
    "GET".to_string()
}

pub fn parse_custom_api_config(raw: &str, base_url: &str) -> Option<CustomApiConfig> {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') {
        let mut cfg: CustomApiConfig = serde_json::from_str(trimmed).ok()?;
        if cfg.url.starts_with('/') {
            cfg.url = format!("{base_url}{}", cfg.url);
        }
        return Some(cfg);
    }
    // pipe format: METHOD|/path|Label:json.path|Label2:path
    let parts: Vec<&str> = trimmed.split('|').collect();
    if parts.len() < 2 {
        return None;
    }
    let method = parts[0].to_uppercase();
    let path = parts[1];
    let url = if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{base_url}{path}")
    };
    let mut fields = std::collections::BTreeMap::new();
    for part in parts.iter().skip(2) {
        if let Some((label, path)) = part.split_once(':') {
            fields.insert(label.trim().to_string(), path.trim().to_string());
        }
    }
    Some(CustomApiConfig {
        url,
        method,
        headers: std::collections::HashMap::new(),
        fields,
    })
}

fn json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        current = current.get(segment)?;
    }
    Some(current)
}

pub async fn fetch_custom_api(client: &Client, base_url: &str, creds_raw: &str) -> Option<Value> {
    let cfg = parse_custom_api_config(creds_raw, base_url)?;
    if !url_allowed_for_health_check(&cfg.url) {
        return None;
    }
    let mut req = match cfg.method.as_str() {
        "POST" => client.post(&cfg.url),
        "PUT" => client.put(&cfg.url),
        _ => client.get(&cfg.url),
    };
    for (k, v) in &cfg.headers {
        req = req.header(k.as_str(), v.as_str());
    }
    let resp = req.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().await.ok()?;
    let mut metrics = serde_json::Map::new();
    metrics.insert("type".into(), json!("custom_api"));
    metrics.insert("tier2".into(), json!(true));
    for (label, path) in cfg.fields.iter().take(6) {
        let val = json_path(&body, path)
            .map(value_to_string)
            .unwrap_or_else(|| "—".to_string());
        let key = label.to_lowercase().replace(' ', "_");
        metrics.insert(key, json!(val));
    }
    if metrics.len() <= 1 {
        metrics.insert("status".into(), json!("ok"));
    }
    Some(Value::Object(metrics))
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pipe_format() {
        let cfg = parse_custom_api_config(
            "GET|/api/status|Online:online|Total:count",
            "http://svc.local",
        )
        .unwrap();
        assert_eq!(cfg.url, "http://svc.local/api/status");
        assert_eq!(cfg.fields.get("Online").unwrap(), "online");
    }
}
