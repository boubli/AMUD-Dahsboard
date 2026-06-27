//! Homarr export JSON importer (apps array from backup/export).

use serde_json::Value;

pub struct HomarrApp {
    pub name: String,
    pub url: String,
    pub icon: String,
    pub category: String,
    pub integration_type: String,
    pub api_key: String,
}

pub fn parse_homarr_export(json: &str) -> Result<Vec<HomarrApp>, String> {
    let root: Value = serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;
    let apps = root
        .get("apps")
        .or_else(|| root.get("data").and_then(|d| d.get("apps")))
        .and_then(|a| a.as_array())
        .ok_or_else(|| "No apps array in Homarr export".to_string())?;
    let mut out = Vec::new();
    for app in apps {
        let name = app
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let url = app
            .get("url")
            .or_else(|| app.get("href"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() || url.is_empty() {
            continue;
        }
        let integration_type = app
            .get("integration")
            .and_then(|i| i.get("type"))
            .or_else(|| app.get("integrationType"))
            .and_then(|v| v.as_str())
            .map(map_homarr_type)
            .unwrap_or_default();
        let api_key = app
            .get("integration")
            .and_then(|i| i.get("apiKey"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let category = app
            .get("category")
            .or_else(|| app.get("group"))
            .and_then(|v| v.as_str())
            .unwrap_or("General")
            .to_string();
        let icon = app
            .get("icon")
            .or_else(|| app.get("iconUrl"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        out.push(HomarrApp {
            name,
            url,
            icon,
            category,
            integration_type,
            api_key,
        });
    }
    Ok(out)
}

fn map_homarr_type(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    crate::integration_registry::map_homepage_widget_type(&lower)
        .unwrap_or(raw)
        .to_string()
}
