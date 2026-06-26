use super::{
    auth_header, get_json, json_str, json_u64, opnsense_basic_auth, parse_pipe_credential,
};
use crate::integrations::format_bytes_short;
use reqwest::Client;
use serde_json::{json, Value};

pub async fn fetch_portainer(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let status = client
        .get(format!("{base}/api/system/status"))
        .header("X-API-Key", key)
        .send()
        .await
        .ok()?;
    if !status.status().is_success() {
        return None;
    }
    let status_json: Value = status.json().await.ok()?;
    let version = json_str(&status_json, &["Version", "version"]);
    let endpoints = client
        .get(format!("{base}/api/endpoints"))
        .header("X-API-Key", key)
        .send()
        .await
        .ok()?
        .json::<Value>()
        .await
        .ok()?;
    let stacks = client
        .get(format!("{base}/api/stacks"))
        .header("X-API-Key", key)
        .send()
        .await
        .ok()?
        .json::<Value>()
        .await
        .ok()?;
    let endpoint_count = endpoints.as_array().map(|a| a.len() as u64).unwrap_or(0);
    let stack_count = stacks.as_array().map(|a| a.len() as u64).unwrap_or(0);
    let (running, stopped) = if let Some(ep) = endpoints.as_array().and_then(|a| a.first()) {
        let ep_id = ep.get("Id").and_then(|v| v.as_u64()).unwrap_or(1);
        let containers = client
            .get(format!(
                "{base}/api/endpoints/{ep_id}/docker/containers/json?all=true"
            ))
            .header("X-API-Key", key)
            .send()
            .await
            .ok()?
            .json::<Value>()
            .await
            .ok()?;
        let mut run = 0u64;
        let mut stop = 0u64;
        if let Some(arr) = containers.as_array() {
            for c in arr {
                let state = c.get("State").and_then(|v| v.as_str()).unwrap_or("");
                if state == "running" {
                    run += 1;
                } else {
                    stop += 1;
                }
            }
        }
        (run, stop)
    } else {
        (0, 0)
    };
    Some(json!({
        "type": "portainer",
        "version": version,
        "endpoints": endpoint_count,
        "stacks": stack_count,
        "containers_running": running,
        "containers_stopped": stopped,
        "containers_total": running + stopped,
        "status": "Online",
    }))
}

pub async fn fetch_opnsense(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = opnsense_basic_auth(api_key);
    let status = get_json(
        client,
        &format!("{base}/api/core/system/status"),
        Some(&auth),
    )
    .await?;
    let resources = get_json(
        client,
        &format!("{base}/api/diagnostics/system/systemResources"),
        Some(&auth),
    )
    .await
    .unwrap_or(json!({}));
    let gateways = get_json(
        client,
        &format!("{base}/api/routes/gateway/status"),
        Some(&auth),
    )
    .await
    .unwrap_or(json!({}));
    let cpu = resources
        .pointer("/cpu/load")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_f64())
        .map(|n| format!("{n:.1}%"))
        .unwrap_or_else(|| "—".to_string());
    let mem_used = resources
        .pointer("/memory/used")
        .or_else(|| resources.get("memory_used"))
        .and_then(|v| {
            v.as_str()
                .map(String::from)
                .or_else(|| v.as_u64().map(format_bytes_short))
        })
        .unwrap_or_else(|| "—".to_string());
    Some(json!({
        "type": "opnsense",
        "version": json_str(&status, &["product_version", "version"]),
        "uptime": json_str(&status, &["uptime"]),
        "cpu": cpu,
        "memory": mem_used,
        "states": json_u64(&resources, &["states", "state_count"]),
        "gateways_up": gateways.as_object().map(|o| {
            o.values().filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("up")).count() as u64
        }).unwrap_or(0),
        "status": "Online",
    }))
}

pub async fn fetch_pfsense(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = format!("{key} {key}");
    let version = get_json(
        client,
        &format!("{base}/api/v2/system/version"),
        Some(&auth),
    )
    .await?;
    let system = get_json(client, &format!("{base}/api/v2/status/system"), Some(&auth))
        .await
        .unwrap_or(json!({}));
    Some(json!({
        "type": "pfsense",
        "version": json_str(&version, &["version", "base", "patch"]),
        "uptime": json_str(&system, &["uptime", "time"]),
        "cpu": system.get("cpu_usage").or_else(|| system.get("cpu"))
            .and_then(|v| v.as_f64().or_else(|| v.as_u64().map(|n| n as f64)))
            .map(|n| format!("{n:.0}%")).unwrap_or_else(|| "—".to_string()),
        "memory": system.get("memory").and_then(|m| m.get("used_percent").or_else(|| m.get("usage")))
            .and_then(|v| v.as_f64().or_else(|| v.as_u64().map(|n| n as f64)))
            .map(|n| format!("{n:.0}%")).unwrap_or_else(|| "—".to_string()),
        "states": json_u64(&system, &["state_count", "states"]),
        "status": "Online",
    }))
}

pub async fn fetch_truenas(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = auth_header(key);
    let info = get_json(client, &format!("{base}/api/v2.0/system/info"), Some(&auth)).await?;
    let pools = get_json(client, &format!("{base}/api/v2.0/pool"), Some(&auth))
        .await
        .unwrap_or(json!([]));
    let mut pool_healthy = 0u64;
    let mut pool_degraded = 0u64;
    let mut used_bytes = 0u64;
    let mut free_bytes = 0u64;
    if let Some(arr) = pools.as_array() {
        for p in arr {
            let status = p.get("status").and_then(|s| s.as_str()).unwrap_or("");
            if status == "ONLINE" {
                pool_healthy += 1;
            } else if !status.is_empty() {
                pool_degraded += 1;
            }
            used_bytes += json_u64(p, &["used"]);
            free_bytes += json_u64(p, &["available"]);
        }
    }
    Some(json!({
        "type": "truenas",
        "version": json_str(&info, &["version", "hostname"]),
        "hostname": json_str(&info, &["hostname"]),
        "pools_healthy": pool_healthy,
        "pools_degraded": pool_degraded,
        "storage_used": format_bytes_short(used_bytes),
        "storage_free": format_bytes_short(free_bytes),
        "status": if pool_degraded == 0 { "Healthy" } else { "Degraded" },
    }))
}

pub async fn fetch_unifi(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let (user, pass) = parse_pipe_credential(api_key)?;
    let base = base_url.trim_end_matches('/');
    let login = client
        .post(format!("{base}/api/auth/login"))
        .json(&json!({ "username": user, "password": pass, "remember": true }))
        .send()
        .await
        .ok()?;
    if !login.status().is_success() {
        return None;
    }
    for prefix in [
        format!("{base}/proxy/network/api/s/default"),
        format!("{base}/api/s/default"),
    ] {
        let health = client
            .get(format!("{prefix}/stat/health"))
            .send()
            .await
            .ok();
        let devices = client
            .get(format!("{prefix}/stat/device"))
            .send()
            .await
            .ok();
        let clients = client.get(format!("{prefix}/stat/sta")).send().await.ok();
        if health.is_none() && devices.is_none() {
            continue;
        }
        let health_json = if let Some(r) = health {
            if r.status().is_success() {
                r.json::<Value>().await.ok()
            } else {
                None
            }
        } else {
            None
        };
        let devices_json = if let Some(r) = devices {
            if r.status().is_success() {
                r.json::<Value>().await.ok()
            } else {
                None
            }
        } else {
            None
        };
        let clients_json = if let Some(r) = clients {
            if r.status().is_success() {
                r.json::<Value>().await.ok()
            } else {
                None
            }
        } else {
            None
        };
        let (wan_status, latency) = health_json
            .as_ref()
            .and_then(|h| h.get("data"))
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .map(|sub| {
                let wan = sub
                    .get("subsystem")
                    .and_then(|s| s.as_str())
                    .unwrap_or("wan");
                let status = sub.get("status").and_then(|s| s.as_str()).unwrap_or("—");
                let lat = sub
                    .get("latency")
                    .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)))
                    .map(|n| format!("{n} ms"))
                    .unwrap_or_else(|| "—".to_string());
                (format!("{wan}: {status}"), lat)
            })
            .unwrap_or(("—".to_string(), "—".to_string()));
        return Some(json!({
            "type": "unifi",
            "wan_status": wan_status,
            "latency": latency,
            "devices": devices_json.as_ref().and_then(|d| d.get("data")).and_then(|a| a.as_array()).map(|a| a.len() as u64).unwrap_or(0),
            "aps_online": devices_json.as_ref().and_then(|d| d.get("data")).and_then(|a| a.as_array()).map(|arr| {
                arr.iter().filter(|d| d.get("state").and_then(|s| s.as_u64()) == Some(1)).count() as u64
            }).unwrap_or(0),
            "clients": clients_json.as_ref().and_then(|c| c.get("data")).and_then(|a| a.as_array()).map(|a| a.len() as u64).unwrap_or(0),
            "status": "Online",
        }));
    }
    None
}

pub async fn fetch_proxmox(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let (user, token) = parse_pipe_credential(api_key)?;
    let base = base_url.trim_end_matches('/');
    let auth = format!("PVEAPIToken={user}={token}");
    let resources = get_json(
        client,
        &format!("{base}/api2/json/cluster/resources"),
        Some(&auth),
    )
    .await?;
    let version = get_json(client, &format!("{base}/api2/json/version"), Some(&auth))
        .await
        .unwrap_or(json!({}));
    let arr = resources.get("data").and_then(|d| d.as_array())?;
    let mut nodes = 0u64;
    let mut vms = 0u64;
    let mut lxcs = 0u64;
    let mut cpu_sum = 0.0f64;
    let mut mem_sum = 0.0f64;
    let mut cpu_count = 0u64;
    for item in arr {
        let kind = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match kind {
            "node" => nodes += 1,
            "qemu" => vms += 1,
            "lxc" => lxcs += 1,
            _ => {}
        }
        if let Some(cpu) = item.get("cpu").and_then(|v| v.as_f64()) {
            cpu_sum += cpu;
            cpu_count += 1;
        }
        if let Some(mem) = item.get("mem").and_then(|v| v.as_f64()) {
            mem_sum += mem;
        }
    }
    let avg_cpu = if cpu_count > 0 {
        format!("{:.0}%", (cpu_sum / cpu_count as f64) * 100.0)
    } else {
        "—".to_string()
    };
    Some(json!({
        "type": "proxmox",
        "version": version.get("data").and_then(|d| d.get("version")).and_then(|v| v.as_str()).unwrap_or("—"),
        "nodes": nodes,
        "vms": vms,
        "lxcs": lxcs,
        "cluster_cpu": avg_cpu,
        "cluster_mem": format_bytes_short(mem_sum as u64),
        "resources": arr.len() as u64,
        "status": "Online",
    }))
}

pub async fn fetch_tailscale(client: &Client, _base_url: &str, api_key: &str) -> Option<Value> {
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = auth_header(key);
    let devices = get_json(
        client,
        "https://api.tailscale.com/api/v2/tailnet/-/devices",
        Some(&auth),
    )
    .await?;
    let arr = devices.get("devices").and_then(|d| d.as_array())?;
    let online = arr
        .iter()
        .filter(|d| d.get("online").and_then(|v| v.as_bool()) == Some(true))
        .count() as u64;
    let exit_nodes = arr
        .iter()
        .filter(|d| {
            d.get("tags")
                .and_then(|t| t.as_array())
                .is_some_and(|tags| tags.iter().any(|tag| tag.as_str() == Some("tag:exit-node")))
        })
        .count() as u64;
    Some(json!({
        "type": "tailscale",
        "devices": arr.len() as u64,
        "devices_online": online,
        "exit_nodes": exit_nodes,
        "version": json_str(&devices, &["clientVersion", "version"]),
        "status": "Online",
    }))
}

pub async fn fetch_netbird(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = auth_header(key);
    let peers = get_json(client, &format!("{base}/api/peers"), Some(&auth)).await?;
    let keys = get_json(client, &format!("{base}/api/setup-keys"), Some(&auth))
        .await
        .unwrap_or(json!([]));
    let peer_arr = peers.as_array().cloned().unwrap_or_default();
    let connected = peer_arr
        .iter()
        .filter(|p| p.get("connected").and_then(|v| v.as_bool()) == Some(true))
        .count() as u64;
    let key_count = keys.as_array().map(|a| a.len() as u64).unwrap_or(0);
    Some(json!({
        "type": "netbird",
        "peers": peer_arr.len() as u64,
        "peers_connected": connected,
        "setup_keys": key_count,
        "status": "Online",
    }))
}
