use doob::db::create_connection;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn setup_test_db() -> Surreal<Db> {
    create_connection(None).await.expect("Failed to create test DB")
}
