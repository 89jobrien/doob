use anyhow::{Context, Result};
use std::path::PathBuf;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;

pub type DbConnection = Surreal<Db>;

/// Opens (or creates) the doob SurrealKV database and initializes the schema.
///
/// `path` overrides the default location (`~/.ctx/doob/db`). Returns errors with
/// the database path in context to aid diagnosis of permission or disk issues.
pub async fn create_connection(path: Option<&str>) -> Result<DbConnection> {
    let db_path = match path {
        Some(p) => PathBuf::from(p),
        None => {
            let mut home = dirs_next::home_dir()
                .context("could not determine home directory — set $HOME or pass --db-path")?;
            home.push(".ctx/doob/db");
            std::fs::create_dir_all(&home).ok();
            home
        }
    };
    let db_path_display = db_path.display().to_string();

    let db = Surreal::new::<SurrealKv>(db_path)
        .await
        .with_context(|| format!("failed to open doob database at {}", db_path_display))?;

    db.use_ns("doob").use_db("doob").await.with_context(|| {
        format!(
            "failed to select doob namespace/database at {}",
            db_path_display
        )
    })?;
    crate::schema::initialize(&db)
        .await
        .with_context(|| format!("failed to initialize doob schema at {}", db_path_display))?;

    Ok(db)
}
