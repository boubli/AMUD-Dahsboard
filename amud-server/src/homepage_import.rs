//! Parse Homepage `services.yaml` / `widgets.yaml` for one-time migration into AMUD.

use crate::integration_registry::map_homepage_widget_type;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ImportedApp {
    pub name: String,
    pub url: String,
    pub icon: String,
    pub description: String,
    pub category: String,
    pub integration_type: String,
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct ImportedWidget {
    pub title: String,
    pub widget_type: String,
    pub content: String,
}

pub fn parse_homepage_services_yaml(yaml: &str) -> Result<Vec<ImportedApp>, String> {
    let root: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| format!("YAML parse error: {e}"))?;
    let mut apps = Vec::new();
    walk_services(&root, "General", &mut apps);
    Ok(apps)
}

fn walk_services(node: &serde_yaml::Value, _category: &str, out: &mut Vec<ImportedApp>) {
    match node {
        serde_yaml::Value::Sequence(items) => {
            for item in items {
                walk_services(item, _category, out);
            }
        }
        serde_yaml::Value::Mapping(map) => {
            for (key, val) in map {
                let group_name = key.as_str().unwrap_or("General");
                if let Some(obj) = val.as_mapping() {
                    if obj.contains_key(serde_yaml::Value::String("href".into()))
                        || obj.contains_key(serde_yaml::Value::String("widget".into()))
                    {
                        if let Some(app) = parse_service_entry(group_name, val) {
                            out.push(app);
                        }
                    } else {
                        walk_services(val, group_name, out);
                    }
                } else {
                    walk_services(val, group_name, out);
                }
            }
        }
        _ => {}
    }
}

fn parse_service_entry(name: &str, val: &serde_yaml::Value) -> Option<ImportedApp> {
    let href = val
        .get("href")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if href.is_empty() {
        return None;
    }
    let icon = val
        .get("icon")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let description = val
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let (integration_type, api_key) = extract_widget(val);
    Some(ImportedApp {
        name: name.to_string(),
        url: href.to_string(),
        icon,
        description,
        category: name.to_string(),
        integration_type,
        api_key,
    })
}

fn extract_widget(val: &serde_yaml::Value) -> (String, String) {
    let widget = val
        .get("widget")
        .or_else(|| val.get("widgets").and_then(|w| w.as_sequence()?.first()));
    let Some(w) = widget else {
        return (String::new(), String::new());
    };
    let wtype = w.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let integration = map_homepage_widget_type(wtype).unwrap_or("").to_string();
    let key = w
        .get("key")
        .or_else(|| w.get("token"))
        .or_else(|| w.get("password"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    (integration, key)
}

pub fn parse_homepage_widgets_yaml(yaml: &str) -> Result<Vec<ImportedWidget>, String> {
    let root: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| format!("YAML parse error: {e}"))?;
    let mut widgets = Vec::new();
    let items = match &root {
        serde_yaml::Value::Sequence(seq) => seq.clone(),
        serde_yaml::Value::Mapping(map) => map.values().cloned().collect(),
        _ => vec![root.clone()],
    };
    for item in items {
        if let serde_yaml::Value::Mapping(map) = item {
            for (k, v) in map {
                let title = k.as_str().unwrap_or("Widget").to_string();
                let wtype = v
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("openmeteo")
                    .to_string();
                let content = serde_json::to_string(&yaml_to_json(&v)).unwrap_or_default();
                widgets.push(ImportedWidget {
                    title,
                    widget_type: wtype,
                    content,
                });
            }
        }
    }
    Ok(widgets)
}

pub fn import_preview_json(
    services_yaml: &str,
    widgets_yaml: Option<&str>,
) -> Result<Value, String> {
    let apps = parse_homepage_services_yaml(services_yaml)?;
    let widgets = widgets_yaml
        .map(parse_homepage_widgets_yaml)
        .transpose()?
        .unwrap_or_default();
    let app_json: Vec<Value> = apps
        .iter()
        .map(|a| {
            json!({
                "name": a.name,
                "url": a.url,
                "icon": a.icon,
                "category": a.category,
                "integration_type": a.integration_type,
                "has_credential": !a.api_key.is_empty(),
            })
        })
        .collect();
    Ok(json!({
        "apps": app_json,
        "app_count": apps.len(),
        "widgets": widgets.len(),
    }))
}

fn yaml_to_json(v: &serde_yaml::Value) -> Value {
    match v {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => json!(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                json!(i)
            } else if let Some(f) = n.as_f64() {
                json!(f)
            } else {
                Value::Null
            }
        }
        serde_yaml::Value::String(s) => json!(s),
        serde_yaml::Value::Sequence(seq) => Value::Array(seq.iter().map(yaml_to_json).collect()),
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, val) in map {
                let key = k.as_str().unwrap_or("key").to_string();
                obj.insert(key, yaml_to_json(val));
            }
            Value::Object(obj)
        }
        serde_yaml::Value::Tagged(t) => yaml_to_json(&t.value),
    }
}

/// Map Docker labels (Homepage convention) to discovered apps.
pub fn apps_from_docker_labels(labels: &BTreeMap<String, String>) -> Vec<ImportedApp> {
    let mut apps = Vec::new();
    let name = labels
        .get("homepage.name")
        .or_else(|| labels.get("homepage.group"))
        .cloned()
        .unwrap_or_else(|| "Docker App".to_string());
    let href = labels.get("homepage.href").cloned().unwrap_or_default();
    if href.is_empty() {
        return apps;
    }
    let icon = labels.get("homepage.icon").cloned().unwrap_or_default();
    let description = labels
        .get("homepage.description")
        .cloned()
        .unwrap_or_default();
    let wtype = labels
        .get("homepage.widget.type")
        .map(|s| s.as_str())
        .unwrap_or("");
    let integration = map_homepage_widget_type(wtype).unwrap_or("").to_string();
    apps.push(ImportedApp {
        name,
        url: href,
        icon,
        description,
        category: labels
            .get("homepage.group")
            .cloned()
            .unwrap_or_else(|| "Docker".to_string()),
        integration_type: integration,
        api_key: labels
            .get("homepage.widget.key")
            .cloned()
            .unwrap_or_default(),
    });
    apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_homepage_services() {
        let yaml = r#"
- Media:
    - Sonarr:
        href: http://sonarr.local
        icon: sonarr.png
        widget:
          type: sonarr
          url: http://sonarr.local
          key: testkey123
"#;
        let apps = parse_homepage_services_yaml(yaml).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].integration_type, "sonarr");
        assert_eq!(apps[0].api_key, "testkey123");
    }

    #[test]
    fn maps_homepage_widget_aliases() {
        use crate::integration_registry::map_homepage_widget_type;
        assert_eq!(map_homepage_widget_type("diskstation"), Some("synology"));
        assert_eq!(map_homepage_widget_type("cloudflared"), Some("cloudflare_tunnel"));
        assert_eq!(map_homepage_widget_type("firefly"), Some("firefly_iii"));
        assert_eq!(map_homepage_widget_type("watchtower"), Some("watchtower"));
    }

    #[test]
    fn parses_diskstation_homepage_service() {
        let yaml = r#"
- NAS:
    - Synology:
        href: http://nas.local
        widget:
          type: diskstation
          url: http://nas.local
          username: admin
          password: secret
"#;
        let apps = parse_homepage_services_yaml(yaml).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].integration_type, "synology");
    }

    #[test]
    fn docker_label_discovery() {
        let mut labels = BTreeMap::new();
        labels.insert("homepage.name".into(), "Radarr".into());
        labels.insert("homepage.href".into(), "http://radarr.local".into());
        labels.insert("homepage.widget.type".into(), "radarr".into());
        let apps = apps_from_docker_labels(&labels);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].integration_type, "radarr");
    }
}
