//! Per-user dashboard boards (layout references).

use rusqlite::{params, Connection};
use serde_json::Value;

pub fn ensure_dashboards_table(conn: &Connection) {
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS dashboards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL DEFAULT 'Default',
            owner_username TEXT NOT NULL DEFAULT 'admin',
            layout_json TEXT NOT NULL DEFAULT '[]',
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
    );
}

pub fn list_dashboards(db: &Connection, owner: &str) -> Vec<Value> {
    ensure_dashboards_table(db);
    let mut out = Vec::new();
    if let Ok(mut stmt) =
        db.prepare("SELECT id, name, owner_username, is_default FROM dashboards WHERE owner_username = ? OR ? = 'admin' ORDER BY is_default DESC, id ASC")
    {
        if let Ok(mut rows) = stmt.query(params![owner, owner]) {
            while let Ok(Some(row)) = rows.next() {
                if let (Ok(id), Ok(name), Ok(user), Ok(def)) = (
                    row.get::<_, i64>(0),
                    row.get::<_, String>(1),
                    row.get::<_, String>(2),
                    row.get::<_, i64>(3),
                ) {
                    out.push(serde_json::json!({
                        "id": id,
                        "name": name,
                        "owner": user,
                        "is_default": def != 0,
                    }));
                }
            }
        }
    }
    out
}

pub fn create_dashboard(db: &Connection, name: &str, owner: &str) -> i64 {
    ensure_dashboards_table(db);
    let _ = db.execute(
        "INSERT INTO dashboards (name, owner_username, layout_json) VALUES (?, ?, '[]')",
        params![name, owner],
    );
    db.last_insert_rowid()
}
