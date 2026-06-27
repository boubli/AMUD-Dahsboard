//! Additional integrations (long-tail Homepage / Homarr parity).

use crate::homelab::{auth_header, get_json};
use reqwest::Client;
use serde_json::{json, Value};

pub async fn fetch_plex_card(client: &Client, base_url: &str, token: &str) -> Option<Value> {
    let sessions_url = format!("{base_url}/status/sessions");
    let resp = client
        .get(&sessions_url)
        .header("X-Plex-Token", token)
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().await.ok()?;
    let streams = body
        .get("MediaContainer")
        .and_then(|m| m.get("Metadata"))
        .and_then(|m| m.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Some(json!({
        "type": "plex",
        "tier2": true,
        "active_streams": streams,
        "status": if streams > 0 { "playing" } else { "idle" },
    }))
}

pub async fn fetch_jellyfin_card(client: &Client, base_url: &str, token: &str) -> Option<Value> {
    let url = format!("{base_url}/Sessions");
    let resp = client
        .get(&url)
        .header("X-Emby-Token", token)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let sessions: Vec<Value> = resp.json().await.ok()?;
    let active = sessions
        .iter()
        .filter(|s| {
            !s.get("UserName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .is_empty()
        })
        .count();
    Some(json!({
        "type": "jellyfin",
        "tier2": true,
        "active_streams": active,
        "status": if active > 0 { "playing" } else { "idle" },
    }))
}

pub async fn fetch_autobrr(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/release"),
        Some(&auth_header(api_key)),
    )
    .await?;
    let count = json.as_array().map(|a| a.len()).unwrap_or(0);
    Some(json!({ "type": "autobrr", "tier2": true, "releases": count, "status": "ok" }))
}

pub async fn fetch_gotify(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/application"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let apps = json.as_array().map(|a| a.len()).unwrap_or(0);
    Some(json!({ "type": "gotify", "tier2": true, "applications": apps, "status": "ok" }))
}

pub async fn fetch_changedetection(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Option<Value> {
    let url = format!("{base_url}/api/v1/watch");
    let resp = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    let watches = json.as_array().map(|a| a.len()).unwrap_or(0);
    Some(json!({ "type": "changedetection", "tier2": true, "watches": watches, "status": "ok" }))
}

pub async fn fetch_prometheus(client: &Client, base_url: &str, _api_key: &str) -> Option<Value> {
    let json = get_json(client, &format!("{base_url}/api/v1/status/config"), None).await?;
    let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    Some(json!({ "type": "prometheus", "tier2": true, "status": status }))
}

pub async fn fetch_openmediavault(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let body = json!({
        "service": "System",
        "method": "getInformation",
        "params": {},
    });
    let resp = client
        .post(format!("{base_url}/rpc.php"))
        .header("X-OPENMEDIAVAULT-SESSIONID", api_key)
        .json(&body)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    let version = json
        .pointer("/response/version")
        .and_then(|v| v.as_str())
        .unwrap_or("—");
    Some(json!({ "type": "openmediavault", "tier2": true, "version": version, "status": "ok" }))
}

pub async fn fetch_freshrss(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let url = format!(
        "{base_url}/api/greader.php/reader/api/0/unread-count?output=json&api_key={api_key}"
    );
    let json = get_json(client, &url, None).await?;
    let unread = json
        .get("unreadcounts")
        .and_then(|u| u.as_array())
        .and_then(|a| a.first())
        .and_then(|e| e.get("count"))
        .and_then(|c| c.as_u64())
        .unwrap_or(0);
    Some(json!({ "type": "freshrss", "tier2": true, "unread": unread, "status": "ok" }))
}

pub async fn fetch_ntfy(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let auth = if api_key.is_empty() {
        None
    } else {
        Some(format!("Bearer {api_key}"))
    };
    let _json = get_json(client, base_url, auth.as_deref()).await?;
    Some(json!({ "type": "ntfy", "tier2": true, "status": "ok" }))
}

pub async fn fetch_coolify(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v1/teams"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let teams = json.as_array().map(|a| a.len()).unwrap_or(0);
    Some(json!({ "type": "coolify", "tier2": true, "teams": teams, "status": "ok" }))
}

pub async fn fetch_aria2(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": "amud",
        "method": "aria2.tellActive",
        "params": [format!("token:{api_key}")],
    });
    let resp = client.post(base_url).json(&body).send().await.ok()?;
    let json: Value = resp.json().await.ok()?;
    let active = json
        .get("result")
        .and_then(|r| r.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Some(json!({ "type": "aria2", "tier2": true, "active": active, "status": "ok" }))
}

pub async fn fetch_kubernetes_summary(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Option<Value> {
    let token = if api_key.is_empty() {
        None
    } else {
        Some(format!("Bearer {api_key}"))
    };
    let nodes = get_json(
        client,
        &format!("{base_url}/api/v1/nodes"),
        token.as_deref(),
    )
    .await?;
    let pods = get_json(client, &format!("{base_url}/api/v1/pods"), token.as_deref()).await?;
    let node_count = nodes
        .get("items")
        .and_then(|i| i.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let pod_count = pods
        .get("items")
        .and_then(|i| i.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Some(json!({
        "type": "kubernetes",
        "tier2": true,
        "nodes": node_count,
        "pods": pod_count,
        "status": "ok",
    }))
}

// Promoted health-only → full cards
pub async fn fetch_gitea_full(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v1/version"),
        Some(&format!("token {api_key}")),
    )
    .await?;
    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("—");
    Some(json!({ "type": "gitea", "tier2": true, "version": version, "status": "ok" }))
}

pub async fn fetch_gitlab_full(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v4/version"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("—");
    Some(json!({ "type": "gitlab", "tier2": true, "version": version, "status": "ok" }))
}

pub async fn fetch_jenkins_full(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/json"),
        Some(&auth_header(api_key)),
    )
    .await?;
    let jobs = json
        .get("jobs")
        .and_then(|j| j.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Some(json!({ "type": "jenkins", "tier2": true, "jobs": jobs, "status": "ok" }))
}

pub async fn fetch_minio_full(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _ = api_key;
    let _json = get_json(client, &format!("{base_url}/minio/health/live"), None).await?;
    Some(json!({ "type": "minio", "tier2": true, "status": "ok" }))
}

pub async fn fetch_github_release(client: &Client, repo: &str, current: &str) -> Option<Value> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let json = get_json(client, &url, None).await?;
    let latest = json.get("tag_name").and_then(|v| v.as_str()).unwrap_or("—");
    let update = latest != current && current != "—";
    Some(json!({
        "type": "github_release",
        "tier2": true,
        "latest": latest,
        "update_available": update,
        "status": if update { "update" } else { "ok" },
    }))
}

pub async fn fetch_dockerhub_release(client: &Client, image: &str, current: &str) -> Option<Value> {
    let url = format!("https://hub.docker.com/v2/repositories/{image}/tags?page_size=1");
    let json = get_json(client, &url, None).await?;
    let latest = json
        .get("results")
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .and_then(|t| t.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("—");
    let update = latest != current && current != "—";
    Some(json!({
        "type": "dockerhub_release",
        "tier2": true,
        "latest": latest,
        "update_available": update,
        "status": if update { "update" } else { "ok" },
    }))
}

fn tier2_ok(integration_type: &str, fields: Value) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("type".into(), json!(integration_type));
    obj.insert("tier2".into(), json!(true));
    obj.insert("status".into(), json!("ok"));
    if let Value::Object(extra) = fields {
        for (k, v) in extra {
            obj.insert(k, v);
        }
    }
    Value::Object(obj)
}

pub async fn fetch_kopia_full(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v1/repo/status"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    Some(tier2_ok("kopia", json!({ "version": status })))
}

pub async fn fetch_headscale_full(
    client: &Client,
    base_url: &str,
    _api_key: &str,
) -> Option<Value> {
    let _json = get_json(client, &format!("{base_url}/health"), None).await?;
    Some(tier2_ok("headscale", json!({})))
}

pub async fn fetch_stash_full(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let body = json!({ "query": "{ stats { scene_count } }" });
    let resp = client
        .post(format!("{base_url}/graphql"))
        .header("ApiKey", api_key)
        .json(&body)
        .send()
        .await
        .ok()?;
    let json: Value = resp.json().await.ok()?;
    let scenes = json
        .pointer("/data/stats/scene_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    Some(tier2_ok("stash", json!({ "scenes": scenes })))
}

pub async fn fetch_healthchecks(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v1/checks/"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let checks = json
        .get("checks")
        .and_then(|c| c.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Some(tier2_ok("healthchecks", json!({ "checks": checks })))
}

pub async fn fetch_gatus(client: &Client, base_url: &str, _api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v1/endpoints/statuses"),
        None,
    )
    .await?;
    let endpoints = json.as_array().map(|a| a.len()).unwrap_or(0);
    Some(tier2_ok("gatus", json!({ "endpoints": endpoints })))
}

pub async fn fetch_scrutiny(client: &Client, base_url: &str, _api_key: &str) -> Option<Value> {
    let json = get_json(client, &format!("{base_url}/api/summary"), None).await?;
    let devices = json
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Some(tier2_ok("scrutiny", json!({ "devices": devices })))
}

pub async fn fetch_uptime_robot(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let body = json!({ "api_key": api_key, "format": "json", "logs": "0" });
    let resp = client
        .post(format!("{base_url}/getMonitors"))
        .json(&body)
        .send()
        .await
        .ok()?;
    let json: Value = resp.json().await.ok()?;
    let monitors = json
        .get("monitors")
        .and_then(|m| m.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Some(tier2_ok("uptime_robot", json!({ "monitors": monitors })))
}

pub async fn fetch_mikrotik(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _ = api_key;
    let _json = get_json(client, &format!("{base_url}/rest/system/resource"), None).await?;
    Some(tier2_ok("mikrotik", json!({})))
}

pub async fn fetch_omada(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _json = get_json(
        client,
        &format!("{base_url}/api/v2/sites"),
        Some(&format!("AccessToken={api_key}")),
    )
    .await?;
    Some(tier2_ok("omada", json!({})))
}

pub async fn fetch_qnap(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _ = api_key;
    let _json = get_json(client, &format!("{base_url}/cgi-bin/authLogin.cgi"), None).await?;
    Some(tier2_ok("qnap", json!({})))
}

pub async fn fetch_gluetun(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let auth = if api_key.is_empty() {
        None
    } else {
        Some(format!("Bearer {api_key}"))
    };
    let json = get_json(
        client,
        &format!("{base_url}/v1/openvpn/status"),
        auth.as_deref(),
    )
    .await?;
    let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    Some(tier2_ok("gluetun", json!({ "vpn": status })))
}

pub async fn fetch_wgeasy(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _json = get_json(
        client,
        &format!("{base_url}/api/wireguard/client"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let clients = _json.as_array().map(|a| a.len()).unwrap_or(0);
    Some(tier2_ok("wgeasy", json!({ "clients": clients })))
}

pub async fn fetch_tubearchivist(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/video/"),
        Some(&format!("Authorization {api_key}")),
    )
    .await?;
    let videos = json
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Some(tier2_ok("tubearchivist", json!({ "videos": videos })))
}

pub async fn fetch_kavita(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/Stats/library-count"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let libraries = json.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    Some(tier2_ok("kavita", json!({ "libraries": libraries })))
}

pub async fn fetch_esphome(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _json = get_json(
        client,
        &format!("{base_url}/devices"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    Some(tier2_ok("esphome", json!({})))
}

pub async fn fetch_octoprint(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/printer"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let state = json
        .get("state")
        .and_then(|s| s.get("text"))
        .and_then(|v| v.as_str())
        .unwrap_or("—");
    Some(tier2_ok("octoprint", json!({ "printer": state })))
}

pub async fn fetch_minecraft(client: &Client, base_url: &str, _api_key: &str) -> Option<Value> {
    let json = get_json(client, &format!("{base_url}/api/status"), None).await?;
    let online = json
        .get("online")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let players = json
        .get("players")
        .and_then(|p| p.get("online"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    Some(tier2_ok(
        "minecraft",
        json!({ "online": if online { "yes" } else { "no" }, "players": players }),
    ))
}

pub async fn fetch_firefly_iii(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v1/about"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let version = json
        .get("data")
        .and_then(|d| d.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("—");
    Some(tier2_ok("firefly_iii", json!({ "version": version })))
}

pub async fn fetch_vikunja(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v1/projects"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let projects = json.as_array().map(|a| a.len()).unwrap_or(0);
    Some(tier2_ok("vikunja", json!({ "projects": projects })))
}

pub async fn fetch_wallos(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _json = get_json(
        client,
        &format!("{base_url}/api/subscriptions"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    Some(tier2_ok("wallos", json!({})))
}

pub async fn fetch_rutorrent(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _ = api_key;
    let _json = get_json(client, &format!("{base_url}/php/rutorrent/RPC2"), None).await?;
    Some(tier2_ok("rutorrent", json!({})))
}

pub async fn fetch_jdownloader(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let _ = api_key;
    let _json = get_json(client, base_url, None).await?;
    Some(tier2_ok("jdownloader", json!({})))
}

pub async fn fetch_zabbix(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let body = json!({
        "jsonrpc": "2.0",
        "method": "host.get",
        "params": { "countOutput": true },
        "auth": api_key,
        "id": 1,
    });
    let resp = client
        .post(format!("{base_url}/api_jsonrpc.php"))
        .json(&body)
        .send()
        .await
        .ok()?;
    let json: Value = resp.json().await.ok()?;
    let hosts = json.get("result").and_then(|v| v.as_str()).unwrap_or("0");
    Some(tier2_ok("zabbix", json!({ "hosts": hosts })))
}

pub async fn fetch_slskd(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/v0/session"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let username = json.get("username").and_then(|v| v.as_str()).unwrap_or("—");
    Some(tier2_ok("slskd", json!({ "user": username })))
}

pub async fn fetch_umami(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let json = get_json(
        client,
        &format!("{base_url}/api/websites"),
        Some(&format!("Bearer {api_key}")),
    )
    .await?;
    let sites = json.as_array().map(|a| a.len()).unwrap_or(0);
    Some(tier2_ok("umami", json!({ "websites": sites })))
}

pub async fn fetch_longtail(
    client: &Client,
    integration_type: &str,
    base_url: &str,
    api_key: &str,
) -> Option<Value> {
    match integration_type {
        "healthchecks" => fetch_healthchecks(client, base_url, api_key).await,
        "gatus" => fetch_gatus(client, base_url, api_key).await,
        "scrutiny" => fetch_scrutiny(client, base_url, api_key).await,
        "uptime_robot" => fetch_uptime_robot(client, base_url, api_key).await,
        "mikrotik" => fetch_mikrotik(client, base_url, api_key).await,
        "omada" => fetch_omada(client, base_url, api_key).await,
        "qnap" => fetch_qnap(client, base_url, api_key).await,
        "gluetun" => fetch_gluetun(client, base_url, api_key).await,
        "wgeasy" => fetch_wgeasy(client, base_url, api_key).await,
        "tubearchivist" => fetch_tubearchivist(client, base_url, api_key).await,
        "kavita" => fetch_kavita(client, base_url, api_key).await,
        "esphome" => fetch_esphome(client, base_url, api_key).await,
        "octoprint" => fetch_octoprint(client, base_url, api_key).await,
        "minecraft" => fetch_minecraft(client, base_url, api_key).await,
        "firefly_iii" => fetch_firefly_iii(client, base_url, api_key).await,
        "vikunja" => fetch_vikunja(client, base_url, api_key).await,
        "wallos" => fetch_wallos(client, base_url, api_key).await,
        "rutorrent" => fetch_rutorrent(client, base_url, api_key).await,
        "jdownloader" => fetch_jdownloader(client, base_url, api_key).await,
        "zabbix" => fetch_zabbix(client, base_url, api_key).await,
        "slskd" => fetch_slskd(client, base_url, api_key).await,
        "umami" => fetch_umami(client, base_url, api_key).await,
        "kopia" => fetch_kopia_full(client, base_url, api_key).await,
        "headscale" => fetch_headscale_full(client, base_url, api_key).await,
        "stash" => fetch_stash_full(client, base_url, api_key).await,
        "channels_dvr" | "calibre_web" | "restic" | "duplicati" | "urbackup" => {
            crate::homelab::fetch_health_integration(client, integration_type, base_url, api_key)
                .await
                .map(|mut v| {
                    if let Some(obj) = v.as_object_mut() {
                        obj.remove("health_only");
                        obj.insert("tier2".into(), json!(true));
                    }
                    v
                })
        }
        _ => None,
    }
}

pub const LONGTAIL_TYPES: &[&str] = &[
    "healthchecks",
    "gatus",
    "scrutiny",
    "uptime_robot",
    "mikrotik",
    "omada",
    "qnap",
    "gluetun",
    "wgeasy",
    "tubearchivist",
    "kavita",
    "esphome",
    "octoprint",
    "minecraft",
    "firefly_iii",
    "vikunja",
    "wallos",
    "rutorrent",
    "jdownloader",
    "zabbix",
    "slskd",
    "umami",
    "kopia",
    "headscale",
    "stash",
    "channels_dvr",
    "calibre_web",
    "restic",
    "duplicati",
    "urbackup",
];

pub fn is_longtail_type(integration_type: &str) -> bool {
    LONGTAIL_TYPES.contains(&integration_type)
}
