use doob::db::{create_connection, DbConnection};
use std::ops::Deref;
use tempfile::TempDir;

pub mod sync_mocks;

pub struct TestDb {
    conn: DbConnection,
    _dir: TempDir,
}

impl Deref for TestDb {
    type Target = DbConnection;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

pub async fn setup_test_db() -> TestDb {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("test.db");
    let path_str = path.to_str().expect("Invalid path").to_string();
    let conn = create_connection(Some(&path_str))
        .await
        .expect("Failed to create test DB");
    TestDb { conn, _dir: dir }
}
