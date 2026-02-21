use rusqlite::Connection;

const SCHEMA_VERSION: u32 = 1;

pub fn create_tables(conn: &Connection) -> Result<(), String> {
    // Wrap DDL + version stamp in one atomic transaction so a mid-crash DB is
    // never left in an indeterminate schema state.
    conn.execute_batch(
        "
        BEGIN;

        CREATE TABLE IF NOT EXISTS providers (
            id             TEXT NOT NULL,
            name           TEXT NOT NULL,
            url            TEXT NOT NULL,
            api_key        TEXT NOT NULL,
            default_model  TEXT NOT NULL DEFAULT '',
            per_cli_models TEXT NOT NULL DEFAULT '{}',
            is_current     INTEGER NOT NULL DEFAULT 0,
            sort_index     INTEGER,
            notes          TEXT,
            created_at     INTEGER NOT NULL,
            PRIMARY KEY (id)
        );

        CREATE TABLE IF NOT EXISTS config_backup (
            app_type        TEXT PRIMARY KEY,
            original_config TEXT NOT NULL,
            backed_up_at    TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT
        );

        COMMIT;
        ",
    )
    .map_err(|e| format!("create_tables failed: {e}"))
}

/// Step-wise migrations keyed by user_version.
/// Each arm must be idempotent for its target version.
/// v0 → v1 is the initial schema (already created by create_tables).
pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| format!("Failed to read user_version: {e}"))?;

    if version < SCHEMA_VERSION {
        // v0 → v1: schema already applied by create_tables above.
        // Future versions add new `if version < N { ... }` blocks here.
        // PRAGMA user_version does not support bound parameters in SQLite.
        // SCHEMA_VERSION is a compile-time const u32 — not user-controlled, safe to format.
        let pragma_sql = format!("PRAGMA user_version = {SCHEMA_VERSION}");
        conn.execute_batch(&pragma_sql)
            .map_err(|e| format!("Failed to set user_version: {e}"))?;
    }

    Ok(())
}
