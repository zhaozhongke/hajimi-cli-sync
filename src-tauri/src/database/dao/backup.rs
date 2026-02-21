use crate::database::{lock_conn, Database};

/// Save the **actual config file content** before overwriting it.
/// Called once per app-type before sync starts. Uses INSERT OR IGNORE so the
/// *first* backup (the pre-switch original) is never overwritten by a retry.
pub fn save_backup(db: &Database, app_type: &str, content: &str) -> Result<(), String> {
    let conn = lock_conn!(db.conn);
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        // INSERT OR IGNORE: never clobber an existing backup for this app.
        // If a backup row already exists the original content is safe.
        "INSERT OR IGNORE INTO config_backup (app_type, original_config, backed_up_at)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![app_type, content, now],
    )
    .map_err(|e| format!("save_backup: {}", e))?;
    Ok(())
}

/// Retrieve a stored config snapshot (used for crash-recovery restore).
pub fn get_backup(db: &Database, app_type: &str) -> Result<Option<String>, String> {
    let conn = lock_conn!(db.conn);
    let mut stmt = conn
        .prepare("SELECT original_config FROM config_backup WHERE app_type = ?1")
        .map_err(|e| format!("prepare get_backup: {}", e))?;
    let mut rows = stmt
        .query_map([app_type], |row| row.get(0))
        .map_err(|e| format!("query get_backup: {}", e))?;
    match rows.next() {
        Some(Ok(v)) => Ok(Some(v)),
        Some(Err(e)) => Err(format!("row get_backup: {}", e)),
        None => Ok(None),
    }
}

/// Delete the backup for one app after a successful restore or sync.
pub fn delete_backup(db: &Database, app_type: &str) -> Result<(), String> {
    let conn = lock_conn!(db.conn);
    conn.execute(
        "DELETE FROM config_backup WHERE app_type = ?1",
        [app_type],
    )
    .map_err(|e| format!("delete_backup: {}", e))?;
    Ok(())
}

/// Delete ALL backups — only called after every restore has succeeded.
pub fn delete_all_backups(db: &Database) -> Result<(), String> {
    let conn = lock_conn!(db.conn);
    conn.execute("DELETE FROM config_backup", [])
        .map_err(|e| format!("delete_all_backups: {}", e))?;
    Ok(())
}

/// List all app-types that have pending backups.
pub fn list_app_types(db: &Database) -> Result<Vec<String>, String> {
    let conn = lock_conn!(db.conn);
    let mut stmt = conn
        .prepare("SELECT app_type FROM config_backup")
        .map_err(|e| format!("prepare list_app_types: {}", e))?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| format!("query list_app_types: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("collect list_app_types: {}", e))
}
