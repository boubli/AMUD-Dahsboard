use crate::models::App;
use crate::settings::SECRET_SETTING_KEYS;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub(crate) fn load_apps_from_db(db: &Connection) -> Vec<App> {
    let mut apps = Vec::new();
    let Ok(mut stmt) = db.prepare(
        "SELECT id, name, url, icon, description, category, node_tag, mac_address FROM apps ORDER BY id DESC",
    ) else {
        return apps;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return apps;
    };
    while let Ok(Some(row)) = rows.next() {
        if let Ok(app) = (|| -> rusqlite::Result<App> {
            Ok(App {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                icon: row.get(3)?,
                description: row.get(4)?,
                category: row.get(5)?,
                node_tag: row.get(6)?,
                mac_address: row.get(7).unwrap_or_else(|_| "".to_string()),
            })
        })() {
            apps.push(app);
        }
    }
    apps
}

pub(crate) async fn with_db<T, F>(db: Arc<Mutex<Connection>>, f: F) -> T
where
    F: FnOnce(&Connection) -> T + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        f(&db)
    })
    .await
    .unwrap_or_else(|e| panic!("database task failed: {e}"))
}

pub(crate) fn refresh_settings_cache(db: &Connection, cache: &RwLock<HashMap<String, String>>) {
    let mut settings = HashMap::new();
    if let Ok(mut stmt) = db.prepare("SELECT key, value FROM settings") {
        if let Ok(mut rows) = stmt.query([]) {
            while let Ok(Some(row)) = rows.next() {
                if let (Ok(key), Ok(value)) = (row.get::<_, String>(0), row.get::<_, String>(1)) {
                    settings.insert(key, value);
                }
            }
        }
    }
    *cache.write().unwrap() = settings;
}

pub(crate) fn secret_setting_configured(settings: &HashMap<String, String>, key: &str) -> bool {
    settings
        .get(key)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub(crate) fn secret_field_placeholder(configured: bool, empty_hint: &str) -> String {
    if configured {
        "Configured — leave blank to keep unchanged".to_string()
    } else {
        empty_hint.to_string()
    }
}

pub(crate) fn setting_value_or_existing(
    db: &Connection,
    key: &str,
    submitted: &str,
) -> Option<String> {
    if SECRET_SETTING_KEYS.contains(&key) && submitted.trim().is_empty() {
        db.query_row(
            "SELECT value FROM settings WHERE key = ?",
            params![key],
            |row| row.get(0),
        )
        .ok()
    } else {
        Some(submitted.to_string())
    }
}

pub(crate) fn telemetry_public_enabled(db: &Connection) -> bool {
    db.query_row(
        "SELECT value FROM settings WHERE key = 'telemetry_public'",
        [],
        |row| row.get::<_, String>(0),
    )
    .map(|v| v == "1")
    .unwrap_or(false)
}

pub(crate) fn load_settings_snapshot(db: &Arc<Mutex<Connection>>) -> HashMap<String, String> {
    let mut settings = HashMap::new();
    let db = db.lock().unwrap();
    if let Ok(mut stmt) = db.prepare("SELECT key, value FROM settings") {
        if let Ok(mut rows) = stmt.query([]) {
            while let Ok(Some(row)) = rows.next() {
                if let (Ok(key), Ok(value)) = (row.get::<_, String>(0), row.get::<_, String>(1)) {
                    settings.insert(key, value);
                }
            }
        }
    }
    settings
}
