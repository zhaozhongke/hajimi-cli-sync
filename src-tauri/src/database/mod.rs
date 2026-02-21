use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub mod dao;
mod schema;

pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

/// Acquire the mutex, recovering from poison (a previous panic inside a lock
/// no longer makes the DB permanently unusable — we take the inner value).
macro_rules! lock_conn {
    ($mutex:expr) => {
        $mutex
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    };
}
pub(crate) use lock_conn;

impl Database {
    pub fn init(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create DB directory: {}", e))?;
        }
        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open SQLite DB: {}", e))?;
        Self::configure(&conn)?;
        Self::apply_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory DB: {}", e))?;
        Self::configure(&conn)?;
        Self::apply_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Per-connection PRAGMAs applied once at open time.
    fn configure(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous  = NORMAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|e| format!("DB configure failed: {}", e))
    }

    fn apply_schema(conn: &Connection) -> Result<(), String> {
        schema::create_tables(conn)?;
        schema::run_migrations(conn)?;
        Ok(())
    }

    /// Returns true if there are any pending crash-recovery backups.
    pub fn has_any_backup(&self) -> Result<bool, String> {
        let conn = lock_conn!(self.conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM config_backup", [], |row| row.get(0))
            .map_err(|e| format!("has_any_backup: {}", e))?;
        Ok(count > 0)
    }
}
