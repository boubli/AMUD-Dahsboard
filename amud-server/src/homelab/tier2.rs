//! Phase 2 integrations — full metric cards.

use super::{
    auth_header, basic_auth, get_json, json_str, json_u64, parse_pipe_credential, post_json,
};
use reqwest::Client;
use serde_json::{json, Value};

macro_rules! online {
    ($type:expr, $($json:tt)*) => {
        Some(json!({ "type": $type, "tier2": true, "status": "Online", $($json)* }))
    };
}

pub async fn fetch_synology(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let (user, pass) = parse_pipe_credential(api_key)?;
    let base = base_url.trim_end_matches('/');
    let login = post_json(
        client,
        &format!("{base}/webapi/auth.cgi"),
        &json!({
            "api": "SYNO.API.Auth",
            "method": "login",
            "version": "3",
            "account": user,
            "passwd": pass,
            "session": "amud",
            "format": "sid"
        }),
        None,
    )
    .await?;
    let sid = login
        .get("data")
        .and_then(|d| d.get("sid"))
        .and_then(|s| s.as_str())?;
    let info = get_json(
        client,
        &format!("{base}/webapi/entry.cgi?api=SYNO.Core.System&method=info&version=1&_sid={sid}"),
        None,
    )
    .await?;
    let storage = get_json(
        client,
        &format!("{base}/webapi/entry.cgi?api=SYNO.Storage.CGI.Storage&method=load_info&version=1&_sid={sid}"),
        None,
    )
    .await
    .unwrap_or(json!({}));
    let volumes = storage
        .pointer("/data/volumes")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(0);
    online!("synology",
        "version": info.pointer("/data/version_string").and_then(|v| v.as_str()).unwrap_or("—"),
        "model": info.pointer("/data/model").and_then(|v| v.as_str()).unwrap_or("—"),
        "volumes": volumes,
        "uptime": info.pointer("/data/uptime").and_then(|v| v.as_u64()).map(|s| format!("{s}s")).unwrap_or_else(|| "—".to_string()),
    )
}

pub async fn fetch_unraid(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let query = json!({
        "query": "{ array { state usedSlots parityCheck { status } } }"
    });
    let resp = post_json(
        client,
        &format!("{base}/graphql"),
        &query,
        Some(&format!("Bearer {key}")),
    )
    .await?;
    let array = resp.pointer("/data/array")?;
    online!("unraid",
        "array_state": json_str(array, &["state"]),
        "used_slots": json_u64(array, &["usedSlots"]),
        "parity_status": array.get("parityCheck").and_then(|p| p.get("status")).and_then(|s| s.as_str()).unwrap_or("—"),
    )
}

pub async fn fetch_dockge(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let mut req = client.get(format!("{base}/api/stacks"));
    if !api_key.trim().is_empty() && api_key.trim() != "none" {
        req = req.header("Authorization", auth_header(api_key.trim()));
    }
    let resp = req.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let stacks: Value = resp.json().await.ok()?;
    let total = stacks.as_array().map(|a| a.len() as u64).unwrap_or(0);
    online!("dockge", "stacks": total, "running": total)
}

pub async fn fetch_nginx_proxy_manager(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Option<Value> {
    let (user, pass) = parse_pipe_credential(api_key)?;
    let base = base_url.trim_end_matches('/');
    let token_resp = post_json(
        client,
        &format!("{base}/api/tokens"),
        &json!({ "identity": user, "secret": pass }),
        None,
    )
    .await?;
    let token = token_resp.get("token").and_then(|t| t.as_str())?;
    let hosts = get_json(
        client,
        &format!("{base}/api/nginx/proxy-hosts"),
        Some(&format!("Bearer {token}")),
    )
    .await?;
    let certs = get_json(
        client,
        &format!("{base}/api/nginx/certificates"),
        Some(&format!("Bearer {token}")),
    )
    .await
    .unwrap_or(json!([]));
    online!("nginx_proxy_manager",
        "proxy_hosts": hosts.as_array().map(|a| a.len() as u64).unwrap_or(0),
        "certificates": certs.as_array().map(|a| a.len() as u64).unwrap_or(0),
    )
}

pub async fn fetch_traefik(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = if api_key.trim().is_empty() || api_key.trim() == "none" {
        None
    } else {
        Some(auth_header(api_key.trim()))
    };
    let overview = get_json(client, &format!("{base}/api/overview"), auth.as_deref()).await?;
    online!("traefik",
        "routers": overview.pointer("/http/routers").and_then(|v| v.as_array()).map(|a| a.len() as u64).unwrap_or(0),
        "services": overview.pointer("/http/services").and_then(|v| v.as_array()).map(|a| a.len() as u64).unwrap_or(0),
        "middlewares": overview.pointer("/http/middlewares").and_then(|v| v.as_array()).map(|a| a.len() as u64).unwrap_or(0),
    )
}

pub async fn fetch_authentik(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = format!("Bearer {key}");
    let users = get_json(
        client,
        &format!("{base}/api/v3/core/users/?page_size=1"),
        Some(&auth),
    )
    .await?;
    let flows = get_json(
        client,
        &format!("{base}/api/v3/flows/instances/?page_size=1"),
        Some(&auth),
    )
    .await
    .unwrap_or(json!({}));
    online!("authentik",
        "users": users.get("pagination").and_then(|p| p.get("count")).and_then(|v| v.as_u64()).unwrap_or(0),
        "flows": flows.get("pagination").and_then(|p| p.get("count")).and_then(|v| v.as_u64()).unwrap_or(0),
    )
}

pub async fn fetch_authelia(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = if api_key.trim().is_empty() || api_key.trim() == "none" {
        None
    } else {
        Some(auth_header(api_key.trim()))
    };
    let health = get_json(client, &format!("{base}/api/health"), auth.as_deref()).await?;
    online!("authelia",
        "status_detail": json_str(&health, &["status"]),
        "version": health.get("version").and_then(|v| v.as_str()).unwrap_or("—"),
    )
}

pub async fn fetch_crowdsec(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = format!("Bearer {key}");
    let alerts = get_json(client, &format!("{base}/v1/alerts"), Some(&auth)).await?;
    let decisions = get_json(client, &format!("{base}/v1/decisions"), Some(&auth))
        .await
        .unwrap_or(json!([]));
    online!("crowdsec",
        "alerts": alerts.as_array().map(|a| a.len() as u64).unwrap_or(0),
        "decisions": decisions.as_array().map(|a| a.len() as u64).unwrap_or(0),
    )
}

pub async fn fetch_node_red(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = if api_key.trim().is_empty() || api_key.trim() == "none" {
        None
    } else {
        Some(auth_header(api_key.trim()))
    };
    let flows = get_json(client, &format!("{base}/flows"), auth.as_deref()).await?;
    let settings = get_json(client, &format!("{base}/settings"), auth.as_deref())
        .await
        .unwrap_or(json!({}));
    online!("node_red",
        "flows": flows.as_array().map(|a| a.len() as u64).unwrap_or(0),
        "version": settings.get("version").and_then(|v| v.as_str()).unwrap_or("—"),
    )
}

pub async fn fetch_zigbee2mqtt(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = if api_key.trim().is_empty() || api_key.trim() == "none" {
        None
    } else {
        Some(auth_header(api_key.trim()))
    };
    let devices = get_json(client, &format!("{base}/api/devices"), auth.as_deref()).await?;
    online!("zigbee2mqtt",
        "devices": devices.as_array().map(|a| a.len() as u64).unwrap_or(0),
    )
}

pub async fn fetch_homeassistant(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = format!("Bearer {key}");
    let states = get_json(client, &format!("{base}/api/states"), Some(&auth)).await?;
    let config = get_json(client, &format!("{base}/api/config"), Some(&auth))
        .await
        .unwrap_or(json!({}));
    let lights = states
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter(|s| {
                    s.get("entity_id")
                        .and_then(|e| e.as_str())
                        .is_some_and(|id| id.starts_with("light."))
                })
                .filter(|s| s.pointer("/state").and_then(|v| v.as_str()) == Some("on"))
                .count() as u64
        })
        .unwrap_or(0);
    let entities = states.as_array().map(|a| a.len() as u64).unwrap_or(0);
    online!("homeassistant",
        "entities": entities,
        "lights_on": lights,
        "version": config.get("version").and_then(|v| v.as_str()).unwrap_or("—"),
        "location": config.get("location_name").and_then(|v| v.as_str()).unwrap_or("—"),
    )
}

pub async fn fetch_emby(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let info = get_json(
        client,
        &format!("{base}/emby/System/Info?api_key={key}"),
        None,
    )
    .await?;
    let sessions = get_json(client, &format!("{base}/emby/Sessions?api_key={key}"), None)
        .await
        .unwrap_or(json!([]));
    online!("emby",
        "version": json_str(&info, &["Version"]),
        "sessions": sessions.as_array().map(|a| a.len() as u64).unwrap_or(0),
        "server_name": json_str(&info, &["ServerName"]),
    )
}

pub async fn fetch_scrypted(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = if api_key.trim().is_empty() || api_key.trim() == "none" {
        None
    } else {
        Some(auth_header(api_key.trim()))
    };
    let plugins = get_json(client, &format!("{base}/api/plugins"), auth.as_deref()).await?;
    online!("scrypted",
        "plugins": plugins.as_array().map(|a| a.len() as u64).unwrap_or(0),
    )
}

async fn fetch_comics_arr(
    client: &Client,
    base_url: &str,
    api_key: &str,
    kind: &str,
    stats_path: &str,
) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let stats = get_json(
        client,
        &format!("{base}{stats_path}"),
        Some(&format!("Bearer {key}")),
    )
    .await?;
    online!(kind,
        "wanted": json_u64(&stats, &["wanted", "missing"]),
        "total": json_u64(&stats, &["total", "series"]),
        "version": json_str(&stats, &["version"]),
    )
}

pub async fn fetch_mylar(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    fetch_comics_arr(client, base_url, api_key, "mylar", "/api?cmd=getVersion").await
}

pub async fn fetch_kapowarr(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let stats = get_json(
        client,
        &format!("{base}/api/stats"),
        Some(&format!("api-key {key}")),
    )
    .await?;
    online!("kapowarr",
        "queue": json_u64(&stats, &["queue", "downloading"]),
        "library": json_u64(&stats, &["library", "total"]),
    )
}

pub async fn fetch_huntarr(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let stats = get_json(
        client,
        &format!("{base}/api/v1/stats"),
        Some(&auth_header(key)),
    )
    .await?;
    online!("huntarr",
        "missing": json_u64(&stats, &["missing", "total_missing"]),
        "apps": json_u64(&stats, &["apps", "configured"]),
    )
}

pub async fn fetch_proxmox_backup(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let (user, token) = parse_pipe_credential(api_key)?;
    let base = base_url.trim_end_matches('/');
    let auth = format!("PBSAPIToken={user}:{token}");
    let stores = get_json(
        client,
        &format!("{base}/api2/json/admin/datastore"),
        Some(&auth),
    )
    .await?;
    let arr = stores.get("data").and_then(|d| d.as_array())?;
    online!("proxmox_backup",
        "datastores": arr.len() as u64,
        "version": "—",
    )
}

pub async fn fetch_technitium(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let zones = get_json(client, &format!("{base}/api/zones/list?token={key}"), None).await?;
    online!("technitium",
        "zones": zones.as_array().map(|a| a.len() as u64).unwrap_or(0),
    )
}

pub async fn fetch_blocky(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = if api_key.trim().is_empty() || api_key.trim() == "none" {
        None
    } else {
        Some(auth_header(api_key.trim()))
    };
    let status = get_json(
        client,
        &format!("{base}/api/blocking/status"),
        auth.as_deref(),
    )
    .await?;
    online!("blocky",
        "blocking": status.get("enabled").and_then(|v| v.as_bool()).map(|b| if b { "on" } else { "off" }).unwrap_or("—"),
    )
}

pub async fn fetch_openwrt(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = if let Some((user, pass)) = parse_pipe_credential(api_key) {
        Some(format!("Basic {}", basic_auth(user, pass)))
    } else if !api_key.trim().is_empty() && api_key.trim() != "none" {
        Some(auth_header(api_key.trim()))
    } else {
        None
    };
    let _ = get_json(
        client,
        &format!("{base}/cgi-bin/luci/admin/status"),
        auth.as_deref(),
    )
    .await?;
    online!("openwrt", "luci": "reachable")
}
