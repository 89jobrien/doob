pub mod schema;

use anyhow::Result;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::Surreal;

// Type alias for the database connection
pub type DbConnection = Surreal<Db>;

pub async fn create_connection(_path: Option<&str>) -> Result<DbConnection> {
    // For now, always use in-memory database
    // File backend will be added when disk space allows RocksDB compilation
    let db = Surreal::new::<Mem>(()).await?;

    db.use_ns("doob").use_db("doob").await?;
    schema::initialize(&db).await?;

    Ok(db)
}
