pub mod schema;

use anyhow::Result;
use std::path::PathBuf;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;

// Type alias for the database connection
pub type DbConnection = Surreal<Db>;

pub async fn create_connection(path: Option<&str>) -> Result<DbConnection> {
    // Use file-based RocksDB storage
    let db_path = path.map(PathBuf::from).unwrap_or_else(|| {
        let mut home = dirs_next::home_dir().expect("Could not find home directory");
        home.push(".ctx/doob/db");
        std::fs::create_dir_all(&home).ok(); // create ~/.ctx/doob/db/ for SurrealKV
        home
    });

    let db = Surreal::new::<SurrealKv>(db_path).await?;

    db.use_ns("doob").use_db("doob").await?;
    schema::initialize(&db).await?;

    Ok(db)
}
