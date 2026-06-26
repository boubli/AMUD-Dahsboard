use super::{auth_header, get_json, json_str, json_u64};
use reqwest::Client;
use serde_json::{json, Value};

pub async fn fetch_grafana(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = auth_header(key);
    let health = get_json(client, &format!("{base}/api/health"), Some(&auth)).await?;
    let dashboards = get_json(
        client,
        &format!("{base}/api/search?type=dash-db"),
        Some(&auth),
    )
    .await
    .unwrap_or(json!([]));
    let datasources = get_json(client, &format!("{base}/api/datasources"), Some(&auth))
        .await
        .unwrap_or(json!([]));
    let org = get_json(client, &format!("{base}/api/org"), Some(&auth))
        .await
        .unwrap_or(json!({}));
    Some(json!({
        "type": "grafana",
        "version": json_str(&health, &["version"]),
        "database": json_str(&health, &["database"]),
        "dashboards": dashboards.as_array().map(|a| a.len() as u64).unwrap_or(0),
        "datasources": datasources.as_array().map(|a| a.len() as u64).unwrap_or(0),
        "organization": json_str(&org, &["name"]),
        "status": "Online",
    }))
}

pub async fn fetch_netdata(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let auth = if api_key.trim().is_empty() || api_key.trim() == "none" {
        None
    } else {
        Some(auth_header(api_key.trim()))
    };
    let info = get_json(client, &format!("{base}/api/v1/info"), auth.as_deref()).await?;
    let alarms = get_json(
        client,
        &format!("{base}/api/v1/alarms?all"),
        auth.as_deref(),
    )
    .await
    .unwrap_or(json!({}));
    let cpu_resp = client
        .get(format!(
            "{base}/api/v1/data?chart=system.cpu&format=json&points=1&group=average"
        ))
        .send()
        .await
        .ok();
    let cpu = if let Some(r) = cpu_resp {
        if r.status().is_success() {
            r.json::<Value>().await.ok().and_then(|d| {
                d.get("data")
                    .and_then(|a| a.as_array())
                    .and_then(|rows| rows.first())
                    .and_then(|row| row.as_array())
                    .and_then(|vals| vals.get(1))
                    .and_then(|v| v.as_f64())
                    .map(|n| format!("{n:.0}%"))
            })
        } else {
            None
        }
    } else {
        None
    }
    .unwrap_or_else(|| "—".to_string());
    Some(json!({
        "type": "netdata",
        "version": json_str(&info, &["version"]),
        "hostname": json_str(&info, &["hostname"]),
        "cpu": cpu,
        "charts": json_u64(&info, &["charts_count", "charts"]),
        "alarms": alarms.get("alarms").and_then(|a| a.as_object()).map(|o| {
            o.values().filter(|a| a.get("status").and_then(|s| s.as_str()) == Some("raised")).count() as u64
        }).unwrap_or(0),
        "status": "Online",
    }))
}

pub async fn fetch_glances(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    for path in ["/api/4/quicklook", "/api/3/quicklook"] {
        let mut req = client.get(format!("{base}{path}"));
        if !api_key.trim().is_empty() && api_key.trim() != "none" {
            req = req.header("Authorization", auth_header(api_key.trim()));
        }
        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(v) = resp.json::<Value>().await {
                    return Some(parse_glances_quicklook(&v));
                }
            }
        }
    }
    None
}

pub(crate) fn parse_glances_quicklook(v: &Value) -> Value {
    let cpu = v
        .get("cpu")
        .and_then(|c| c.get("total").or_else(|| c.get("percent")))
        .and_then(|x| x.as_f64().or_else(|| x.as_u64().map(|n| n as f64)))
        .map(|n| format!("{n:.0}%"))
        .unwrap_or_else(|| "—".to_string());
    let mem = v
        .get("mem")
        .or_else(|| v.get("memory"))
        .and_then(|m| m.get("percent").or_else(|| m.get("used")))
        .and_then(|x| x.as_f64().or_else(|| x.as_u64().map(|n| n as f64)))
        .map(|n| format!("{n:.0}%"))
        .unwrap_or_else(|| "—".to_string());
    let load = v
        .pointer("/load/min1")
        .or_else(|| v.get("load"))
        .and_then(|x| x.as_f64())
        .map(|n| format!("{n:.2}"))
        .unwrap_or_else(|| "—".to_string());
    json!({
        "type": "glances",
        "cpu": cpu,
        "memory": mem,
        "load": load,
        "status": "Online",
    })
}

pub async fn fetch_beszel(client: &Client, base_url: &str, api_key: &str) -> Option<Value> {
    let base = base_url.trim_end_matches('/');
    let key = api_key.trim();
    if key.is_empty() {
        return None;
    }
    let auth = auth_header(key);
    let systems = get_json(
        client,
        &format!("{base}/api/collections/systems/records"),
        Some(&auth),
    )
    .await?;
    let items = systems
        .get("items")
        .and_then(|i| i.as_array())
        .cloned()
        .unwrap_or_default();
    let total = systems
        .get("totalItems")
        .and_then(|v| v.as_u64())
        .unwrap_or(items.len() as u64);
    let online = items
        .iter()
        .filter(|s| s.get("status").and_then(|v| v.as_str()) == Some("up"))
        .count() as u64;
    let avg_cpu = if items.is_empty() {
        "—".to_string()
    } else {
        let sum: f64 = items
            .iter()
            .filter_map(|s| {
                s.get("info")
                    .and_then(|i| i.get("cpu"))
                    .and_then(|c| c.as_f64())
            })
            .sum();
        let count = items
            .iter()
            .filter(|s| s.get("info").and_then(|i| i.get("cpu")).is_some())
            .count();
        if count > 0 {
            format!("{:.0}%", sum / count as f64)
        } else {
            "—".to_string()
        }
    };
    Some(json!({
        "type": "beszel",
        "systems": total,
        "systems_up": online,
        "systems_down": total.saturating_sub(online),
        "avg_cpu": avg_cpu,
        "status": "Online",
    }))
}
