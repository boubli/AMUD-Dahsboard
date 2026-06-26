use super::{auth_header, basic_auth, get_json, json_str, json_u64, parse_pipe_credential};
use crate::integrations::format_bytes_short;
use reqwest::Client;
use serde_json::{json, Value};

pub async fn fetch_paperless(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = format!("Token {key}");
    let stats = get_json(client, &format!("{base}/api/statistics/"), Some(&auth)).await?;
    let correspondents = get_json(
        client,
        &format!("{base}/api/correspondents/?page_size=1"),
        Some(&auth),
    )
    .await
    .unwrap_or(json!({}));
    let tags = get_json(
        client,
        &format!("{base}/api/tags/?page_size=1"),
        Some(&auth),
    )
    .await
    .unwrap_or(json!({}));
    Some(json!({
        "type": "paperless",
        "documents": json_u64(&stats, &["documents_total", "documents"]),
        "inbox": json_u64(&stats, &["inbox_total", "inbox"]),
        "correspondents": correspondents.get("count").and_then(|v| v.as_u64()).unwrap_or(0),
        "tags": tags.get("count").and_then(|v| v.as_u64()).unwrap_or(0),
        "storage": stats.get("character_count").and_then(|v| v.as_u64()).map(format_bytes_short).unwrap_or_else(|| "—".to_string()),
        "status": "Online",
    }))
}

pub async fn fetch_mealie(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = auth_header(key);
    let about = get_json(client, &format!("{base}/api/households/about"), Some(&auth)).await?;
    let recipes = client
        .get(format!("{base}/api/recipes?page=1&perPage=1"))
        .header("Authorization", &auth)
        .send()
        .await
        .ok()?;
    let recipe_count = recipes
        .headers()
        .get("x-total-count")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let users = get_json(client, &format!("{base}/api/users"), Some(&auth))
        .await
        .unwrap_or(json!([]));
    Some(json!({
        "type": "mealie",
        "version": json_str(&about, &["version"]),
        "recipes": recipe_count,
        "users": users.as_array().map(|a| a.len() as u64).unwrap_or(0),
        "status": "Online",
    }))
}

pub async fn fetch_nextcloud(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let (user, pass) = parse_pipe_credential(api_key)?;
    let base = base_url.trim_end_matches('/');
    let basic = basic_auth(user, pass);
    let resp = client
        .get(format!(
            "{base}/ocs/v2.php/apps/serverinfo/api/v1/info?format=json"
        ))
        .header("Authorization", format!("Basic {basic}"))
        .header("OCS-APIRequest", "true")
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    let data = json.pointer("/ocs/data")?;
    Some(json!({
        "type": "nextcloud",
        "version": data.pointer("/nextcloud/version").and_then(|v| v.as_str()).unwrap_or("—"),
        "users_active": data.pointer("/activeUsers/last24hours").and_then(|v| v.as_u64()).unwrap_or(0),
        "users_online": data.pointer("/activeUsers/last5minutes").and_then(|v| v.as_u64()).unwrap_or(0),
        "memory": data.pointer("/server/mem_total").and_then(|v| v.as_u64()).map(format_bytes_short).unwrap_or_else(|| "—".to_string()),
        "free_space": data.pointer("/nextcloud/system").and_then(|s| s.as_array()).and_then(|arr| {
            arr.iter().find(|e| e.get("id").and_then(|id| id.as_str()) == Some("freespace"))
                .and_then(|e| e.get("value")).and_then(|v| v.as_str()).map(String::from)
        }).unwrap_or_else(|| "—".to_string()),
        "status": "Online",
    }))
}

pub async fn fetch_vaultwarden(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = auth_header(key);
    let config = get_json(client, &format!("{base}/api/config"), None).await?;
    let users = client
        .get(format!("{base}/admin/users"))
        .header("Authorization", &auth)
        .send()
        .await
        .ok();
    let user_count = if let Some(r) = users {
        if r.status().is_success() {
            r.json::<Value>()
                .await
                .ok()
                .and_then(|v| v.as_array().map(|a| a.len() as u64))
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    Some(json!({
        "type": "vaultwarden",
        "version": json_str(&config, &["version"]),
        "users": user_count,
        "organizations": 0,
        "server": config.get("environment").and_then(|e| e.get("serverName")).and_then(|v| v.as_str()).unwrap_or("—"),
        "status": "Online",
    }))
}
