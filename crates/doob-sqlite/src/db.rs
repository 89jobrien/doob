use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Thread-safe wrapper around a SQLite connection.
#[derive(Clone)]
pub struct SqliteConnection {
    inner: Arc<Mutex<Connection>>,
}

impl SqliteConnection {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let parent = path
            .parent()
            .context("database path has no parent directory")?;
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;

        let conn =
            Connection::open(path).with_context(|| format!("failed to open {}", path.display()))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Run a closure with exclusive access to the underlying connection.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self
            .inner
            .lock()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?;
        f(&conn)
    }
}

/// Create a connection at the default path (`~/.ctx/doob/doob.db`)
/// or at a caller-specified path.
pub fn create_connection(path: Option<&str>) -> Result<SqliteConnection> {
    let db_path = match path {
        Some(p) => PathBuf::from(p),
        None => {
            let mut home = dirs_next::home_dir().context("could not determine home directory")?;
            home.push(".ctx/doob/doob.db");
            home
        }
    };

    let conn = SqliteConnection::open(&db_path)?;
    crate::schema::initialize(&conn)?;
    Ok(conn)
}
