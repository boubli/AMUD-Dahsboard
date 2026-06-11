use crate::models::AppState;
use std::sync::Arc;
use std::time::Duration;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct HaState {
    entity_id: String,
    state: String,
    // attributes: serde_json::Value,
}

#[derive(Clone, serde::Serialize, Default)]
pub struct SmartHomeTelemetry {
    pub lights_on: usize,
    pub switches_on: usize,
    pub avg_temp: Option<f64>,
}

pub async fn start_ha_polling(state: Arc<AppState>) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let settings = state.settings_cache.read().unwrap().clone();
        let ha_url = settings.get("ha_url").cloned().unwrap_or_default();
        let ha_token = settings.get("ha_token").cloned().unwrap_or_default();

        if ha_url.is_empty() || ha_token.is_empty() {
            continue;
        }

        let api_url = format!("{}/api/states", ha_url.trim_end_matches('/'));
        
        let mut telemetry = SmartHomeTelemetry::default();
        let mut temp_sum = 0.0;
        let mut temp_count = 0;

        if let Ok(resp) = client
            .get(&api_url)
            .header("Authorization", format!("Bearer {}", ha_token))
            .send()
            .await
        {
            if let Ok(states) = resp.json::<Vec<HaState>>().await {
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
            }
        }

        if temp_count > 0 {
            telemetry.avg_temp = Some((temp_sum / temp_count as f64 * 10.0).round() / 10.0);
        }

        let mut lock = state.smart_home_telemetry.write().unwrap();
        *lock = telemetry;
    }
}
