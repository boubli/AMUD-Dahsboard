//! Background poll coordinator — warms integration cache for dashboard apps only.

use crate::db::load_apps_from_db;
use crate::integration_registry::ttl_for_type;
use crate::models::{App, AppState};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

const COORDINATOR_INTERVAL_SECS: u64 = 45;

pub fn start_integration_coordinator(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(COORDINATOR_INTERVAL_SECS)).await;
            let accept_invalid = {
                let cache = state.settings_cache.read().unwrap();
                cache
                    .get("accept_invalid_certs")
                    .map(|s| s == "1")
                    .unwrap_or(false)
            };
            let apps: Vec<App> = {
                let db = state.db.lock().unwrap();
                load_apps_from_db(&db)
            };
            let integrated: Vec<App> = apps
                .into_iter()
                .filter(|a| !a.integration_type.is_empty() && !a.api_key.is_empty())
                .collect();
            if integrated.is_empty() {
                continue;
            }
            let stagger = COORDINATOR_INTERVAL_SECS / integrated.len().max(1) as u64;
            for (i, app) in integrated.iter().enumerate() {
                if i > 0 {
                    sleep(Duration::from_millis(stagger * 250)).await;
                }
                let app = app.clone();
                let cache = state.integration_cache.clone();
                let clients = state.http_clients.clone();
                let ttl = ttl_for_type(&app.integration_type);
                let _ = cache
                    .get_or_fetch(app.id, ttl, || {
                        let a = app.clone();
                        async move {
                            crate::integrations::fetch_integration_data_uncached(
                                &a,
                                accept_invalid,
                                &clients,
                            )
                            .await
                        }
                    })
                    .await;
            }
        }
    });
}
