use crate::database::{lock_conn, Database};

pub fn get(db: &Database, key: &str) -> Result<Option<String>, String> {
    let conn = lock_conn!(db.conn);
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| format!("prepare settings get: {}", e))?;
    let mut rows = stmt
        .query_map([key], |row| row.get(0))
        .map_err(|e| format!("query settings get: {}", e))?;
    match rows.next() {
        Some(Ok(v)) => Ok(Some(v)),
        Some(Err(e)) => Err(format!("row settings get: {}", e)),
        None => Ok(None),
    }
}

pub fn set(db: &Database, key: &str, value: &str) -> Result<(), String> {
    let conn = lock_conn!(db.conn);
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [key, value],
    )
    .map_err(|e| format!("settings set: {}", e))?;
    Ok(())
}
