//! Background poll coordinator — warms integration cache for dashboard apps only.

use crate::db::load_apps_from_db;
use crate::models::{App, AppState};
use crate::settings::{feeds_enabled, setting_u64_bounded};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

const IDLE_BACKOFF_SECS: u64 = 300;

pub(crate) fn should_poll_integration(integration_type: &str, feeds_on: bool) -> bool {
    feeds_on || integration_type != "rss"
}

pub fn start_integration_coordinator(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            let settings = state.settings_cache.read().unwrap().clone();
            let interval = setting_u64_bounded(
                &settings,
                "integration_coordinator_interval_secs",
                45,
                15,
                600,
            );
            let feeds_on = feeds_enabled(&settings);
            let accept_invalid = settings
                .get("accept_invalid_certs")
                .map(|s| s == "1")
                .unwrap_or(false);

            let apps: Vec<App> = {
                let db = state.db.lock().unwrap();
                load_apps_from_db(&db)
            };
            let integrated: Vec<App> = apps
                .into_iter()
                .filter(|a| {
                    !a.integration_type.is_empty()
                        && !a.api_key.is_empty()
                        && should_poll_integration(&a.integration_type, feeds_on)
                })
                .collect();

            if integrated.is_empty() {
                sleep(Duration::from_secs(IDLE_BACKOFF_SECS)).await;
                continue;
            }

            sleep(Duration::from_secs(interval)).await;

            let stagger = interval / integrated.len().max(1) as u64;
            for (i, app) in integrated.iter().enumerate() {
                if i > 0 {
                    sleep(Duration::from_millis(stagger * 250)).await;
                }
                let app = app.clone();
                let cache = state.integration_cache.clone();
                let clients = state.http_clients.clone();
                let ttl = Duration::from_secs(setting_u64_bounded(
                    &settings,
                    "integration_cache_ttl_secs",
                    45,
                    5,
                    600,
                ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_rss_when_feeds_disabled() {
        assert!(!should_poll_integration("rss", false));
        assert!(should_poll_integration("rss", true));
        assert!(should_poll_integration("radarr", false));
    }
}
