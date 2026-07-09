use crate::models::AppState;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct HaState {
    entity_id: String,
    state: String,
}

#[derive(Clone, serde::Serialize, Default)]
pub struct SmartHomeTelemetry {
    pub lights_on: usize,
    pub switches_on: usize,
    pub avg_temp: Option<f64>,
}

#[derive(Deserialize)]
struct HaTemplateSummary {
    lights_on: usize,
    switches_on: usize,
    #[serde(default)]
    temps: Vec<f64>,
}

/// Renders a compact JSON summary on the Home Assistant host instead of downloading all states.
const HA_SUMMARY_TEMPLATE: &str = r#"{"lights_on":{{ states.light | selectattr('state','eq','on') | list | count }},"switches_on":{{ states.switch | selectattr('state','eq','on') | list | count }},"temps":[{% for s in states.sensor if 'temperature' in s.entity_id and s.state | is_number %}{{ s.state | float }}{% if not loop.last %},{% endif %}{% endfor %}]}"#;

fn summary_from_template_json(raw: &str) -> Option<SmartHomeTelemetry> {
    let parsed: HaTemplateSummary = serde_json::from_str(raw.trim()).ok()?;
    let avg_temp = if parsed.temps.is_empty() {
        None
    } else {
        let sum: f64 = parsed.temps.iter().sum();
        Some(((sum / parsed.temps.len() as f64) * 10.0).round() / 10.0)
    };
    Some(SmartHomeTelemetry {
        lights_on: parsed.lights_on,
        switches_on: parsed.switches_on,
        avg_temp,
    })
}

async fn poll_ha_template(
    client: &reqwest::Client,
    ha_url: &str,
    ha_token: &str,
) -> Option<SmartHomeTelemetry> {
    let api_url = format!("{}/api/template", ha_url.trim_end_matches('/'));
    let resp = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", ha_token))
        .json(&serde_json::json!({ "template": HA_SUMMARY_TEMPLATE }))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().await.ok()?;
    summary_from_template_json(&body)
}

async fn poll_ha_states_fallback(
    client: &reqwest::Client,
    ha_url: &str,
    ha_token: &str,
) -> SmartHomeTelemetry {
    let mut telemetry = SmartHomeTelemetry::default();
    let api_url = format!("{}/api/states", ha_url.trim_end_matches('/'));
    let Ok(resp) = client
        .get(api_url)
        .header("Authorization", format!("Bearer {}", ha_token))
        .send()
        .await
    else {
        return telemetry;
    };
    let Ok(states) = resp.json::<Vec<HaState>>().await else {
        return telemetry;
    };

    let mut temp_sum = 0.0;
    let mut temp_count = 0;
    for s in states {
        if s.entity_id.starts_with("light.") && s.state == "on" {
            telemetry.lights_on += 1;
        } else if s.entity_id.starts_with("switch.") && s.state == "on" {
            telemetry.switches_on += 1;
        } else if s.entity_id.starts_with("sensor.") && s.entity_id.contains("temperature") {
            if let Ok(temp) = s.state.parse::<f64>() {
                temp_sum += temp;
                temp_count += 1;
            }
        }
    }
    if temp_count > 0 {
        telemetry.avg_temp = Some((temp_sum / temp_count as f64 * 10.0).round() / 10.0);
    }
    telemetry
}

pub async fn start_ha_polling(state: Arc<AppState>) {
    loop {
        tokio::time::sleep(Duration::from_secs(15)).await;

        let settings = state.settings_cache.read().unwrap().clone();
        let ha_url = settings.get("ha_url").cloned().unwrap_or_default();
        let ha_token = settings.get("ha_token").cloned().unwrap_or_default();

        if ha_url.is_empty() || ha_token.is_empty() {
            continue;
        }

        let accept_invalid = settings
            .get("accept_invalid_certs")
            .map(|v| v == "1")
            .unwrap_or(false);
        let client =
            crate::http_client::select_http_client(&state.http_clients, accept_invalid).clone();

        let telemetry = match poll_ha_template(&client, &ha_url, &ha_token).await {
            Some(summary) => summary,
            None => poll_ha_states_fallback(&client, &ha_url, &ha_token).await,
        };

        *state.smart_home_telemetry.write().unwrap() = telemetry;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_template_summary_json() {
        let summary =
            summary_from_template_json(r#"{"lights_on":3,"switches_on":1,"temps":[21.5,22.0]}"#)
                .unwrap();
        assert_eq!(summary.lights_on, 3);
        assert_eq!(summary.switches_on, 1);
        assert_eq!(summary.avg_temp, Some(21.8));
    }
}
