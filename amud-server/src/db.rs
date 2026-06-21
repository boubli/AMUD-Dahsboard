use crate::audit::record_audit;
use crate::auth::{hash_password, verify_password};
use crate::models::{App, Webhook, WolDevice};
use crate::security::mask_webhook_url;
use crate::settings::SECRET_SETTING_KEYS;
use axum::http::HeaderMap;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub(crate) fn load_apps_from_db(db: &Connection) -> Vec<App> {
    let mut apps = Vec::new();
    let Ok(mut stmt) = db.prepare(
        "SELECT id, name, url, icon, description, category, node_tag, mac_address, integration_type, api_key, sort_order, card_span FROM apps ORDER BY sort_order ASC, id ASC",
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
                icon: row.get(3).unwrap_or_else(|_| "".to_string()),
                description: row.get(4).unwrap_or_else(|_| "".to_string()),
                category: row.get(5)?,
                node_tag: row.get(6)?,
                mac_address: row.get(7).unwrap_or_else(|_| "".to_string()),
                integration_type: row.get(8).unwrap_or_else(|_| "".to_string()),
                api_key: {
                    let raw_key = row.get::<_, String>(9).unwrap_or_default();
                    crate::secrets::decrypt_value(&raw_key).unwrap_or(raw_key)
                },
                sort_order: row.get(10).unwrap_or(0),
                card_span: row.get(11).unwrap_or_else(|_| "1x1".to_string()),
            })
        })() {
            apps.push(app);
        }
    }
    apps
}

pub(crate) fn load_wol_devices_from_db(db: &Connection) -> Vec<WolDevice> {
    let mut devices = Vec::new();
    let Ok(mut stmt) = db.prepare(
        "SELECT id, name, mac_address, ip_address, icon FROM wol_devices ORDER BY id DESC",
    ) else {
        return devices;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return devices;
    };
    while let Ok(Some(row)) = rows.next() {
        if let Ok(dev) = (|| -> rusqlite::Result<WolDevice> {
            Ok(WolDevice {
                id: row.get(0)?,
                name: row.get(1)?,
                mac_address: row.get(2)?,
                ip_address: row.get(3).unwrap_or_else(|_| "".to_string()),
                icon: row.get(4).unwrap_or_else(|_| "".to_string()),
            })
        })() {
            devices.push(dev);
        }
    }
    devices
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
                    let value = crate::secrets::decrypt_setting_from_db(&key, &value);
                    settings.insert(key, value);
                }
            }
        }
    }
    *cache.write().unwrap() = settings;
}

pub(crate) fn upsert_setting(db: &Connection, key: &str, value: &str) {
    let stored = crate::secrets::encrypt_setting_for_db(key, value);
    db.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, stored],
    )
    .ok();
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
            |row| row.get::<_, String>(0),
        )
        .ok()
        .map(|stored| crate::secrets::decrypt_setting_from_db(key, &stored))
        .filter(|v| !v.is_empty())
    } else {
        Some(submitted.to_string())
    }
}

pub(crate) fn telemetry_public_from_cache(settings: &HashMap<String, String>) -> bool {
    settings
        .get("telemetry_public")
        .map(|v| v.as_str())
        .unwrap_or("0")
        == "1"
}

pub(crate) fn default_category_name(db: &Connection) -> String {
    db.query_row(
        "SELECT name FROM categories ORDER BY sort_order ASC, name ASC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .unwrap_or_else(|_| "General".to_string())
}

/// Ensures apps.category always references an existing category name.
pub(crate) fn resolve_app_category(db: &Connection, category: &str) -> String {
    let name = category.trim();
    if name.is_empty() {
        return default_category_name(db);
    }
    let exists = db
        .query_row(
            "SELECT 1 FROM categories WHERE name = ? LIMIT 1",
            params![name],
            |_| Ok(()),
        )
        .is_ok();
    if exists {
        name.to_string()
    } else {
        default_category_name(db)
    }
}

pub(crate) fn load_categories(db: &Connection) -> Vec<(i64, String)> {
    let mut categories = Vec::new();
    let Ok(mut stmt) =
        db.prepare("SELECT id, name FROM categories ORDER BY sort_order ASC, name ASC")
    else {
        return categories;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return categories;
    };
    while let Ok(Some(row)) = rows.next() {
        if let (Ok(id), Ok(name)) = (row.get::<_, i64>(0), row.get::<_, String>(1)) {
            categories.push((id, name));
        }
    }
    categories
}

pub(crate) fn load_app_name_urls(db: &Connection) -> Vec<(String, String)> {
    let mut apps = Vec::new();
    let Ok(mut stmt) = db.prepare("SELECT name, url FROM apps") else {
        return apps;
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }) else {
        return apps;
    };
    for app in rows.flatten() {
        apps.push(app);
    }
    apps
}

pub(crate) fn load_categories_json(db: &Connection) -> Vec<serde_json::Value> {
    let mut categories = Vec::new();
    let Ok(mut stmt) = db.prepare(
        "SELECT id, name, color, sort_order FROM categories ORDER BY sort_order ASC, name ASC",
    ) else {
        return categories;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return categories;
    };
    while let Ok(Some(row)) = rows.next() {
        if let (Ok(id), Ok(name), Ok(color), Ok(sort_order)) = (
            row.get::<_, i64>(0),
            row.get::<_, String>(1),
            row.get::<_, String>(2),
            row.get::<_, i64>(3),
        ) {
            categories.push(serde_json::json!({
                "id": id,
                "name": name,
                "color": color,
                "sort_order": sort_order
            }));
        }
    }
    categories
}

pub(crate) fn load_users_json(db: &Connection) -> Vec<serde_json::Value> {
    let mut users = Vec::new();
    let Ok(mut stmt) = db.prepare("SELECT id, username, role FROM users") else {
        return users;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return users;
    };
    while let Ok(Some(row)) = rows.next() {
        if let (Ok(id), Ok(username), Ok(role)) = (
            row.get::<_, i64>(0),
            row.get::<_, String>(1),
            row.get::<_, String>(2),
        ) {
            users.push(serde_json::json!({ "id": id, "username": username, "role": role }));
        }
    }
    users
}

pub(crate) fn load_active_webhooks(db: &Connection) -> Vec<Webhook> {
    let mut list = Vec::new();
    let Ok(mut stmt) = db
        .prepare("SELECT id, name, url, event_types, is_active FROM webhooks WHERE is_active = 1")
    else {
        return list;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return list;
    };
    while let Ok(Some(row)) = rows.next() {
        if let (Ok(id), Ok(name), Ok(url), Ok(event_types), Ok(is_active)) = (
            row.get::<_, i64>(0),
            row.get::<_, String>(1),
            row.get::<_, String>(2),
            row.get::<_, String>(3),
            row.get::<_, i32>(4),
        ) {
            list.push(Webhook {
                id,
                name,
                url,
                event_types,
                is_active,
            });
        }
    }
    list
}

pub(crate) fn load_active_webhooks_for_event(db: &Connection, event_type: &str) -> Vec<Webhook> {
    load_active_webhooks(db)
        .into_iter()
        .filter(|wh| wh.event_types.split(',').any(|e| e.trim() == event_type))
        .collect()
}

pub(crate) fn load_webhooks_json(db: &Connection) -> Vec<serde_json::Value> {
    let mut list = Vec::new();
    let Ok(mut stmt) =
        db.prepare("SELECT id, name, url, event_types, is_active FROM webhooks ORDER BY id DESC")
    else {
        return list;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return list;
    };
    while let Ok(Some(row)) = rows.next() {
        if let (Ok(id), Ok(name), Ok(url), Ok(event_types), Ok(is_active)) = (
            row.get::<_, i64>(0),
            row.get::<_, String>(1),
            row.get::<_, String>(2),
            row.get::<_, String>(3),
            row.get::<_, i32>(4),
        ) {
            list.push(serde_json::json!({
                "id": id,
                "name": name,
                "url": mask_webhook_url(&url),
                "event_types": event_types,
                "is_active": is_active
            }));
        }
    }
    list
}

pub(crate) fn fetch_wol_device_mac_address(db: &Connection, id: i64) -> Option<String> {
    db.query_row(
        "SELECT mac_address FROM wol_devices WHERE id = ?",
        params![id],
        |row| row.get(0),
    )
    .ok()
}

pub(crate) fn insert_wol_device(
    db: &Connection,
    name: &str,
    mac: &str,
    ip: &str,
    icon: &str,
) -> Result<i64, rusqlite::Error> {
    db.execute(
        "INSERT INTO wol_devices (name, mac_address, ip_address, icon) VALUES (?, ?, ?, ?)",
        params![name, mac, ip, icon],
    )?;
    Ok(db.last_insert_rowid())
}

pub(crate) fn delete_wol_device(db: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    db.execute("DELETE FROM wol_devices WHERE id = ?", params![id])?;
    Ok(())
}

pub(crate) fn fetch_webhook_by_id(db: &Connection, id: i64) -> Option<(String, String)> {
    db.query_row(
        "SELECT name, url FROM webhooks WHERE id = ?",
        params![id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .ok()
}

pub(crate) struct LoginDbResult {
    pub success: bool,
    pub role: String,
    pub must_change_password: bool,
}

pub(crate) fn process_login(db: &Connection, username: &str, password: &str) -> LoginDbResult {
    let fail = LoginDbResult {
        success: false,
        role: String::new(),
        must_change_password: false,
    };
    let Ok(mut stmt) = db.prepare("SELECT password_hash, role FROM users WHERE username = ?")
    else {
        return fail;
    };
    let auth_res = stmt.query_row(params![username], |row| {
        let pwhash: String = row.get(0)?;
        let role: String = row.get(1)?;
        let (verified, needs_rehash) = verify_password(&pwhash, password);
        Ok((verified, needs_rehash, role))
    });
    let Ok((true, needs_rehash, role)) = auth_res else {
        return fail;
    };
    if needs_rehash {
        let upgraded = hash_password(password);
        db.execute(
            "UPDATE users SET password_hash = ? WHERE username = ?",
            params![upgraded, username],
        )
        .ok();
    }
    let must_change = db
        .query_row(
            "SELECT value FROM settings WHERE key = 'admin_must_change_password'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map(|v| v == "1")
        .unwrap_or(false);
    LoginDbResult {
        success: true,
        role,
        must_change_password: must_change,
    }
}

pub(crate) enum CategoryDeleteError {
    LastCategory,
    NotFound,
    DeleteFailed,
}

pub(crate) fn delete_category_by_id(db: &Connection, id: i64) -> Result<(), CategoryDeleteError> {
    let cat_count: i64 = db
        .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
        .unwrap_or(0);
    if cat_count <= 1 {
        return Err(CategoryDeleteError::LastCategory);
    }
    let old_name: String = db
        .query_row(
            "SELECT name FROM categories WHERE id = ?",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or_default();
    if old_name.is_empty() {
        return Err(CategoryDeleteError::NotFound);
    }
    let fallback: String = db
        .query_row(
            "SELECT name FROM categories WHERE id != ? ORDER BY sort_order ASC, name ASC LIMIT 1",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| default_category_name(db));
    db.execute(
        "UPDATE apps SET category = ? WHERE category = ?",
        params![fallback, old_name],
    )
    .ok();
    if db
        .execute("DELETE FROM categories WHERE id = ?", params![id])
        .is_err()
    {
        return Err(CategoryDeleteError::DeleteFailed);
    }
    Ok(())
}

pub(crate) fn update_category_by_id(
    db: &Connection,
    id: i64,
    name: &str,
    color: &str,
    sort_order: i64,
) {
    let old_name: String = db
        .query_row(
            "SELECT name FROM categories WHERE id = ?",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or_default();
    if db
        .execute(
            "UPDATE categories SET name = ?, color = ?, sort_order = ? WHERE id = ?",
            params![name, color, sort_order, id],
        )
        .is_ok()
        && !old_name.is_empty()
        && old_name != name
    {
        db.execute(
            "UPDATE apps SET category = ? WHERE category = ?",
            params![name, old_name],
        )
        .ok();
    }
}

pub(crate) enum DeleteUserError {
    NotFound,
    SelfDelete,
    LastAdmin,
    Failed,
}

pub(crate) fn delete_user_by_id(
    db: &Connection,
    id: i64,
    self_username: Option<&str>,
    admin_user: &str,
    headers: &HeaderMap,
) -> Result<(), DeleteUserError> {
    let target_role: String = db
        .query_row("SELECT role FROM users WHERE id = ?", params![id], |row| {
            row.get(0)
        })
        .unwrap_or_default();
    if target_role.is_empty() {
        return Err(DeleteUserError::NotFound);
    }
    if let Some(username) = self_username {
        let self_id: Option<i64> = db
            .query_row(
                "SELECT id FROM users WHERE username = ?",
                params![username],
                |row| row.get(0),
            )
            .ok();
        if self_id == Some(id) {
            return Err(DeleteUserError::SelfDelete);
        }
    }
    if target_role == "Admin" {
        let admin_count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM users WHERE role = 'Admin'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if admin_count <= 1 {
            return Err(DeleteUserError::LastAdmin);
        }
    }
    if db
        .execute("DELETE FROM users WHERE id = ?", params![id])
        .is_err()
    {
        return Err(DeleteUserError::Failed);
    }
    record_audit(
        db,
        admin_user,
        "user_delete",
        &format!("id:{id}"),
        "user removed",
        headers,
    );
    Ok(())
}

pub(crate) fn record_audit_blocking(
    db: &Connection,
    headers: &HeaderMap,
    actor: &str,
    action: &str,
    target: &str,
    details: &str,
) {
    record_audit(db, actor, action, target, details, headers);
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

pub(crate) fn update_app_order(db: &Connection, ids: &[i64]) -> Result<(), rusqlite::Error> {
    let tx = db.unchecked_transaction()?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE apps SET sort_order = ? WHERE id = ?",
            params![i as i64, id],
        )?;
    }
    tx.commit()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE categories (id INTEGER PRIMARY KEY, name TEXT, color TEXT, sort_order INTEGER);
             CREATE TABLE apps (id INTEGER PRIMARY KEY, name TEXT, url TEXT, icon TEXT, description TEXT, category TEXT, node_tag TEXT, mac_address TEXT);
             INSERT INTO categories (name, color, sort_order) VALUES ('General', '#000', 0);
             INSERT INTO categories (name, color, sort_order) VALUES ('Media', '#111', 1);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn resolve_app_category_uses_existing_name() {
        let db = test_db();
        assert_eq!(resolve_app_category(&db, "Media"), "Media");
    }

    #[test]
    fn resolve_app_category_falls_back_for_unknown() {
        let db = test_db();
        assert_eq!(resolve_app_category(&db, "Missing"), "General");
        assert_eq!(resolve_app_category(&db, ""), "General");
    }
}
