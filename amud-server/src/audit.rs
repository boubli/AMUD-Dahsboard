use axum::http::HeaderMap;
use rusqlite::{params, Connection, ErrorCode};

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

fn audit_table_has_column(db: &Connection, column: &str) -> bool {
    let Ok(mut stmt) = db.prepare("PRAGMA table_info(audit_log)") else {
        return false;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return false;
    };
    while let Ok(Some(row)) = rows.next() {
        if row.get::<_, String>(1).ok().as_deref() == Some(column) {
            return true;
        }
    }
    false
}

/// Ensures audit_log exists and has all expected columns (handles partial upgrades).
pub(crate) fn ensure_audit_log_schema(db: &Connection) -> Result<(), rusqlite::Error> {
    ensure_audit_log_table(db)?;
    if !audit_table_has_column(db, "details") {
        let _ = db.execute(
            "ALTER TABLE audit_log ADD COLUMN details TEXT NOT NULL DEFAULT ''",
            [],
        );
    }
    if !audit_table_has_column(db, "client_ip") {
        db.execute(
            "ALTER TABLE audit_log ADD COLUMN client_ip TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }
    Ok(())
}

pub(crate) fn audit_health_check(db: &Connection) -> Result<i64, String> {
    ensure_audit_log_schema(db).map_err(|e| format!("schema setup failed: {e}"))?;
    db.query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
        .map_err(|e| format!("count query failed: {e}"))
}

pub(crate) fn record_audit(
    db: &Connection,
    username: &str,
    action: &str,
    target: &str,
    details: &str,
    headers: &HeaderMap,
) {
    if let Err(e) = ensure_audit_log_schema(db) {
        eprintln!("[AUDIT] failed to ensure audit_log schema: {e}");
        return;
    }

    let ip = client_ip(headers);
    let sql = "INSERT INTO audit_log (created_at, username, action, target, details, client_ip)
         VALUES (datetime('now'), ?1, ?2, ?3, ?4, ?5)";

    let mut last_err = None;
    for attempt in 0..3 {
        match db.execute(sql, params![username, action, target, details, ip]) {
            Ok(_) => return,
            Err(e) if e.sqlite_error_code() == Some(ErrorCode::DatabaseBusy) && attempt < 2 => {
                std::thread::yield_now();
                last_err = Some(e);
            }
            Err(e) => {
                eprintln!("[AUDIT] insert failed (action={action}, user={username}): {e}");
                return;
            }
        }
    }
    if let Some(e) = last_err {
        eprintln!("[AUDIT] insert failed after retries (action={action}): {e}");
    }
}

pub(crate) fn list_recent_audit(
    db: &Connection,
    limit: i64,
) -> Result<Vec<serde_json::Value>, String> {
    ensure_audit_log_schema(db).map_err(|e| format!("audit log unavailable: {e}"))?;

    let mut out = Vec::new();
    let mut stmt = db
        .prepare(
            "SELECT id, created_at, username, action, target, details, client_ip
         FROM audit_log ORDER BY id DESC LIMIT ?",
        )
        .map_err(|e| format!("audit log query prepare failed: {e}"))?;
    let mut rows = stmt
        .query(params![limit])
        .map_err(|e| format!("audit log query failed: {e}"))?;
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("audit log read failed: {e}"))?
    {
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
    Ok(out)
}
