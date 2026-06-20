use axum::http::HeaderMap;
use rusqlite::{params, Connection, ErrorCode};
use std::thread;
use std::time::Duration;

use crate::security::client_ip;

/// Idempotent schema guard for databases upgraded from releases before audit_log existed.
pub(crate) fn ensure_audit_log_table(db: &Connection) -> Result<(), rusqlite::Error> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS audit_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        username TEXT NOT NULL,
        action TEXT NOT NULL,
        target TEXT NOT NULL,
        details TEXT NOT NULL DEFAULT '',
        client_ip TEXT NOT NULL DEFAULT ''
    );",
        [],
    )?;
    let _ = db.execute(
        "CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created_at);",
        [],
    );
    Ok(())
}

pub(crate) fn record_audit(
    db: &Connection,
    username: &str,
    action: &str,
    target: &str,
    details: &str,
    headers: &HeaderMap,
) {
    if let Err(e) = ensure_audit_log_table(db) {
        eprintln!("[AUDIT] failed to ensure audit_log table: {e}");
        return;
    }

    let ip = client_ip(headers);
    let sql = "INSERT INTO audit_log (created_at, username, action, target, details, client_ip)
         VALUES (datetime('now'), ?1, ?2, ?3, ?4, ?5)";

    let mut last_err = None;
    for attempt in 0..3 {
        match db.execute(sql, params![username, action, target, details, ip]) {
            Ok(_) => {
                eprintln!(
                    "[AUDIT] user={username} action={action} target={target} details={details} ip={ip}"
                );
                return;
            }
            Err(e) if e.sqlite_error_code() == Some(ErrorCode::DatabaseBusy) && attempt < 2 => {
                thread::sleep(Duration::from_millis(25 * (attempt as u64 + 1)));
                last_err = Some(e);
            }
            Err(e) => {
                eprintln!("[AUDIT] insert failed: {e}");
                return;
            }
        }
    }
    if let Some(e) = last_err {
        eprintln!("[AUDIT] insert failed after retries: {e}");
    }
}

pub(crate) fn list_recent_audit(db: &Connection, limit: i64) -> Vec<serde_json::Value> {
    if ensure_audit_log_table(db).is_err() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let Ok(mut stmt) = db.prepare(
        "SELECT id, created_at, username, action, target, details, client_ip
         FROM audit_log ORDER BY id DESC LIMIT ?",
    ) else {
        return out;
    };
    let Ok(mut rows) = stmt.query(params![limit]) else {
        return out;
    };
    while let Ok(Some(row)) = rows.next() {
        out.push(serde_json::json!({
            "id": row.get::<_, i64>(0).unwrap_or(0),
            "created_at": row.get::<_, String>(1).unwrap_or_default(),
            "username": row.get::<_, String>(2).unwrap_or_default(),
            "action": row.get::<_, String>(3).unwrap_or_default(),
            "target": row.get::<_, String>(4).unwrap_or_default(),
            "details": row.get::<_, String>(5).unwrap_or_default(),
            "client_ip": row.get::<_, String>(6).unwrap_or_default(),
        }));
    }
    out
}
