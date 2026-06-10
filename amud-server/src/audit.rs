use axum::http::HeaderMap;
use rusqlite::{params, Connection};

use crate::security::client_ip;

pub(crate) fn record_audit(
    db: &Connection,
    username: &str,
    action: &str,
    target: &str,
    details: &str,
    headers: &HeaderMap,
) {
    let ip = client_ip(headers);
    db.execute(
        "INSERT INTO audit_log (created_at, username, action, target, details, client_ip)
         VALUES (datetime('now'), ?, ?, ?, ?, ?)",
        params![username, action, target, details, ip],
    )
    .ok();
    eprintln!(
        "[AUDIT] user={} action={} target={} details={} ip={}",
        username, action, target, details, ip
    );
}

pub(crate) fn list_recent_audit(db: &Connection, limit: i64) -> Vec<serde_json::Value> {
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
