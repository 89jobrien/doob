use anyhow::{Context, Result};
use std::path::PathBuf;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;

pub type DbConnection = Surreal<Db>;

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

    let db = Surreal::new::<SurrealKv>(db_path).await?;

    db.use_ns("doob").use_db("doob").await?;
    crate::schema::initialize(&db).await?;

    Ok(db)
}
