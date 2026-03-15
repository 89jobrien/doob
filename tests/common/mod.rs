use doob::db::{create_connection, DbConnection};

pub mod sync_mocks;

pub async fn setup_test_db() -> DbConnection {
    create_connection(None)
        .await
        .expect("Failed to create test DB")
}
