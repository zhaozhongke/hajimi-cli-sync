use crate::database::{lock_conn, Database};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderRecord {
    pub id: String,
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub default_model: String,
    pub per_cli_models: String, // JSON string: Record<string,string>
    pub is_current: bool,
    pub sort_index: Option<i64>,
    pub notes: Option<String>,
    pub created_at: i64, // Unix seconds
}

// ── shared row-mapper ────────────────────────────────────────────────────────

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderRecord> {
    Ok(ProviderRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        api_key: row.get(3)?,
        default_model: row.get(4)?,
        per_cli_models: row.get(5)?,
        is_current: row.get::<_, i64>(6)? != 0,
        sort_index: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
    })
}

// ── public API ───────────────────────────────────────────────────────────────

pub fn get_all(db: &Database) -> Result<Vec<ProviderRecord>, String> {
    let conn = lock_conn!(db.conn);
    let mut stmt = conn
        .prepare(
            "SELECT id, name, url, api_key, default_model, per_cli_models, is_current,
                    sort_index, notes, created_at
             FROM providers
             ORDER BY COALESCE(sort_index, 999999), created_at ASC",
        )
        .map_err(|e| format!("prepare get_all: {e}"))?;
    let rows = stmt
        .query_map([], map_row)
        .map_err(|e| format!("query get_all: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("collect get_all: {e}"))
}

pub fn get_current(db: &Database) -> Result<Option<ProviderRecord>, String> {
    let conn = lock_conn!(db.conn);
    let mut stmt = conn
        .prepare(
            "SELECT id, name, url, api_key, default_model, per_cli_models, is_current,
                    sort_index, notes, created_at
             FROM providers WHERE is_current = 1 LIMIT 1",
        )
        .map_err(|e| format!("prepare get_current: {e}"))?;
    let mut rows = stmt
        .query_map([], map_row)
        .map_err(|e| format!("query get_current: {e}"))?;
    match rows.next() {
        Some(Ok(r)) => Ok(Some(r)),
        Some(Err(e)) => Err(format!("row get_current: {e}")),
        None => Ok(None),
    }
}

/// Upsert — atomic INSERT OR REPLACE so there is no check-then-act race.
/// `is_current` and `created_at` are preserved from the existing row on UPDATE
/// via the ON CONFLICT replacement semantics (the caller must supply correct
/// values when inserting; for updates we re-read the stored is_current first).
pub fn save(db: &Database, provider: &ProviderRecord) -> Result<(), String> {
    let conn = lock_conn!(db.conn);

    // Read the stored is_current so an edit never accidentally clears it.
    let existing_is_current: Option<i64> = conn
        .query_row(
            "SELECT is_current FROM providers WHERE id = ?1",
            [&provider.id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("save read existing: {e}"))?;

    let is_current = existing_is_current.unwrap_or(0);

    conn.execute(
        "INSERT INTO providers
             (id, name, url, api_key, default_model, per_cli_models,
              is_current, sort_index, notes, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT(id) DO UPDATE SET
             name          = excluded.name,
             url           = excluded.url,
             api_key       = excluded.api_key,
             default_model = excluded.default_model,
             per_cli_models= excluded.per_cli_models,
             sort_index    = excluded.sort_index,
             notes         = excluded.notes",
        rusqlite::params![
            provider.id,
            provider.name,
            provider.url,
            provider.api_key,
            provider.default_model,
            provider.per_cli_models,
            is_current,
            provider.sort_index,
            provider.notes,
            provider.created_at,
        ],
    )
    .map_err(|e| format!("save upsert: {e}"))?;
    Ok(())
}

/// Atomically transfer `is_current` to `id` in a single transaction.
/// Uses parameterised statements inside an explicit transaction — no format! interpolation.
pub fn set_current(db: &Database, id: &str) -> Result<(), String> {
    let conn = lock_conn!(db.conn);

    // Verify the target exists before we mutate anything.
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM providers WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| format!("set_current pre-check: {e}"))?;
    if exists == 0 {
        return Err(format!("Provider not found: {id}"));
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("set_current begin: {e}"))?;
    tx.execute("UPDATE providers SET is_current = 0", [])
        .map_err(|e| format!("set_current clear: {e}"))?;
    tx.execute("UPDATE providers SET is_current = 1 WHERE id = ?1", [id])
        .map_err(|e| format!("set_current set: {e}"))?;
    tx.commit().map_err(|e| format!("set_current commit: {e}"))
}

/// Delete a provider. Refuses if it is currently active.
/// Uses parameterised statements inside an explicit transaction — no format! interpolation.
pub fn delete(db: &Database, id: &str) -> Result<(), String> {
    let conn = lock_conn!(db.conn);

    // Pre-check: is_current guard (parameterised).
    let is_current: i64 = conn
        .query_row(
            "SELECT COALESCE(is_current, 0) FROM providers WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("delete pre-check: {e}"))?
        .unwrap_or(0);

    if is_current != 0 {
        return Err(
            "Cannot delete the active provider — switch to another provider first.".to_string(),
        );
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("delete begin: {e}"))?;
    tx.execute("DELETE FROM providers WHERE id = ?1", [id])
        .map_err(|e| format!("delete execute: {e}"))?;
    tx.commit().map_err(|e| format!("delete commit: {e}"))
}

/// Batch-update sort_index in a single transaction.
pub fn reorder(db: &Database, ids: &[String]) -> Result<(), String> {
    let conn = lock_conn!(db.conn);
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("reorder begin: {e}"))?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE providers SET sort_index = ?1 WHERE id = ?2",
            rusqlite::params![i as i64, id],
        )
        .map_err(|e| format!("reorder update {id}: {e}"))?;
    }
    tx.commit().map_err(|e| format!("reorder commit: {e}"))
}

#[allow(dead_code)]
pub fn count(db: &Database) -> Result<i64, String> {
    let conn = lock_conn!(db.conn);
    conn.query_row("SELECT COUNT(*) FROM providers", [], |row| row.get(0))
        .map_err(|e| format!("count: {e}"))
}
