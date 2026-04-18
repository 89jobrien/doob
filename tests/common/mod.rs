use doob::adapters::TodoRepositoryImpl;
use doob::db::{create_connection, DbConnection};
use doob::models::{Note, Todo};
use doob::ports::TodoRepository;
use anyhow::Result;
use async_trait::async_trait;
use std::ops::Deref;
use tempfile::TempDir;

pub mod sync_mocks;

pub struct TestDb {
    conn: DbConnection,
    repo: TodoRepositoryImpl,
    _dir: TempDir,
}

impl Deref for TestDb {
    type Target = DbConnection;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

#[async_trait]
impl TodoRepository for TestDb {
    async fn create_todos(
        &self,
        todos: Vec<(
            String,
            String,
            u8,
            Option<String>,
            Option<String>,
            Vec<String>,
        )>,
    ) -> Result<Vec<Todo>> {
        self.repo.create_todos(todos).await
    }

    async fn get_todo(&self, record_id: &str) -> Result<Option<Todo>> {
        self.repo.get_todo(record_id).await
    }

    async fn list_todos(
        &self,
        status: Option<&str>,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Todo>> {
        self.repo.list_todos(status, project, limit).await
    }

    async fn update_todo(
        &self,
        record_id: &str,
        priority: Option<u8>,
        status: Option<&str>,
        project: Option<&str>,
        tags: Option<Vec<String>>,
        content: Option<&str>,
    ) -> Result<Todo> {
        self.repo
            .update_todo(record_id, priority, status, project, tags, content)
            .await
    }

    async fn delete_todo(&self, record_id: &str) -> Result<()> {
        self.repo.delete_todo(record_id).await
    }

    async fn complete_todo(&self, record_id: &str) -> Result<()> {
        self.repo.complete_todo(record_id).await
    }

    async fn undo_todo(&self, record_id: &str) -> Result<()> {
        self.repo.undo_todo(record_id).await
    }

    async fn search_todos(&self, query: &str, project: Option<&str>) -> Result<Vec<Todo>> {
        self.repo.search_todos(query, project).await
    }

    async fn get_todo_stats(&self) -> Result<serde_json::Value> {
        self.repo.get_todo_stats().await
    }

    async fn create_notes(
        &self,
        notes: Vec<(String, Option<String>, Option<String>, Vec<String>)>,
    ) -> Result<Vec<Note>> {
        self.repo.create_notes(notes).await
    }

    async fn get_note(&self, record_id: &str) -> Result<Option<Note>> {
        self.repo.get_note(record_id).await
    }

    async fn list_notes(&self, project: Option<&str>, limit: Option<usize>) -> Result<Vec<Note>> {
        self.repo.list_notes(project, limit).await
    }

    async fn delete_note(&self, record_id: &str) -> Result<()> {
        self.repo.delete_note(record_id).await
    }

    async fn search_notes(&self, query: &str, project: Option<&str>) -> Result<Vec<Note>> {
        self.repo.search_notes(query, project).await
    }

    async fn execute_raw_query(&self, query: &str) -> Result<serde_json::Value> {
        self.repo.execute_raw_query(query).await
    }
}

pub async fn setup_test_db() -> TestDb {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("test.db");
    let path_str = path.to_str().expect("Invalid path").to_string();
    let conn = create_connection(Some(&path_str))
        .await
        .expect("Failed to create test DB");
    let repo = TodoRepositoryImpl::new(conn.clone());
    TestDb {
        conn,
        repo,
        _dir: dir,
    }
}
