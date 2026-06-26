use super::{get_json, json_str, json_u64, parse_pipe_credential};
use reqwest::Client;
use serde_json::{json, Value};

pub async fn fetch_deluge(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let password = api_key.trim();
    if password.is_empty() {
        return None;
    }
    let rpc_url = format!("{base}/json");
    let login_resp = client
        .post(&rpc_url)
        .json(&json!({ "method": "auth.login", "params": [password], "id": 1 }))
        .send()
        .await
        .ok()?;
    let login_json: Value = login_resp.json().await.ok()?;
    if login_json.get("result").and_then(|v| v.as_bool()) != Some(true) {
        return None;
    }
    let torrents_resp = client
        .post(&rpc_url)
        .json(&json!({ "method": "web.get_torrents_status", "params": [{}], "id": 2 }))
        .send()
        .await
        .ok()?;
    let torrents_json: Value = torrents_resp.json().await.ok()?;
    let torrents = torrents_json.get("result").and_then(|r| r.as_object());
    let mut downloading = 0u64;
    let mut seeding = 0u64;
    if let Some(map) = torrents {
        for (_, t) in map {
            match t.get("state").and_then(|s| s.as_str()).unwrap_or("") {
                "Downloading" | "Active" => downloading += 1,
                "Seeding" => seeding += 1,
                _ => {}
            }
        }
    }
    Some(json!({
        "type": "deluge",
        "downloading": downloading,
        "seeding": seeding,
        "torrents": torrents.map(|m| m.len() as u64).unwrap_or(0),
        "free_space": "—",
        "status": "Online",
    }))
}

pub async fn fetch_navidrome(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let (user, pass) = parse_pipe_credential(api_key)?;
    let base = base_url.trim_end_matches('/');
    let params = format!(
        "u={}&p={}&v=1.16.0&c=amud&f=json",
        urlencoding::encode(user),
        urlencoding::encode(pass)
    );
    let ping = get_json(client, &format!("{base}/rest/ping.view?{params}"), None).await?;
    if ping
        .pointer("/subsonic-response/status")
        .and_then(|s| s.as_str())
        != Some("ok")
    {
        return None;
    }
    let indexes = get_json(
        client,
        &format!("{base}/rest/getIndexes.view?{params}"),
        None,
    )
    .await?;
    let artists = indexes
        .pointer("/subsonic-response/indexes/index")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|idx| idx.get("artist"))
                .filter_map(|a| a.as_array())
                .map(|a| a.len() as u64)
                .sum::<u64>()
        })
        .unwrap_or(0);
    Some(json!({
        "type": "navidrome",
        "artists": artists,
        "version": indexes.pointer("/subsonic-response/version").and_then(|v| v.as_str()).unwrap_or("—"),
        "status": "Online",
    }))
}

pub async fn fetch_komga(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let libraries = client
        .get(format!("{base}/api/v1/libraries"))
        .header("X-API-Key", key)
        .send()
        .await
        .ok()?;
    if !libraries.status().is_success() {
        return None;
    }
    let libs: Value = libraries.json().await.ok()?;
    let series_resp = client
        .get(format!("{base}/api/v1/series?page=0&size=1"))
        .header("X-API-Key", key)
        .send()
        .await
        .ok()?;
    let books_resp = client
        .get(format!("{base}/api/v1/books?page=0&size=1"))
        .header("X-API-Key", key)
        .send()
        .await
        .ok()?;
    let series_total = series_resp
        .headers()
        .get("x-total-count")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let book_total = books_resp
        .headers()
        .get("x-total-count")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Some(json!({
        "type": "komga",
        "libraries": libs.as_array().map(|a| a.len() as u64).unwrap_or(0),
        "series": series_total,
        "books": book_total,
        "status": "Online",
    }))
}

pub async fn fetch_photoprism(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let cred = api_key.trim();
    if cred.is_empty() {
        return None;
    }
    let auth = if cred.contains('|') {
        let (user, pass) = parse_pipe_credential(cred)?;
        format!("Basic {}", super::basic_auth(user, pass))
    } else {
        format!("Basic {cred}")
    };
    let config = get_json(client, &format!("{base}/api/v1/config"), Some(&auth)).await?;
    let index = get_json(client, &format!("{base}/api/v1/index"), Some(&auth))
        .await
        .unwrap_or(json!({}));
    Some(json!({
        "type": "photoprism",
        "version": config.get("version").and_then(|v| v.as_str()).unwrap_or("—"),
        "photos": json_u64(&config, &["count", "photos"]),
        "videos": index.get("videos").and_then(|v| v.as_u64()).unwrap_or(0),
        "albums": json_u64(&config, &["albums"]),
        "index_status": json_str(&index, &["status"]),
        "status": "Online",
    }))
}
