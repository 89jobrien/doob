//! Test-only fixtures shared across `doob-core`'s own test modules.
//!
//! Not exposed outside `#[cfg(test)]` — kept in `doob-core` (rather than
//! reusing `doob`'s `TestDb`) so this crate's tests don't depend on the
//! `doob` binary crate.

use crate::models::Todo;
use crate::ports::TodoRepository;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;

/// A `HashMap`-backed `TodoRepository`, keyed by `Todo::uuid`.
///
/// Only the methods `DoobCheckpointStore` actually calls
/// (`get_todo_by_uuid`, `create_todos`, `update_todo`) are implemented;
/// every other `TodoRepository` method is `unimplemented!()` since nothing
/// under test calls them.
#[derive(Default)]
pub struct InMemoryTodoRepository {
    todos: Mutex<HashMap<String, Todo>>,
}

impl InMemoryTodoRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl TodoRepository for InMemoryTodoRepository {
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
        let mut store = self.todos.lock().unwrap();
        let mut created = Vec::with_capacity(todos.len());
        for (content, uuid, priority, project, file_path, tags) in todos {
            let now = Utc::now();
            let todo = Todo {
                id: None,
                uuid: uuid.clone(),
                content,
                status: crate::models::todo::TodoStatus::Pending,
                priority,
                created_at: now,
                updated_at: now,
                completed_at: None,
                due_date: None,
                project,
                project_path: None,
                file_path,
                tags,
                metadata: None,
                blocks: Vec::new(),
                blocked_by: Vec::new(),
            };
            store.insert(uuid, todo.clone());
            created.push(todo);
        }
        Ok(created)
    }

    async fn get_todo(&self, _record_id: &str) -> Result<Option<Todo>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn list_todos(
        &self,
        _status: Option<&str>,
        _project: Option<&str>,
        _limit: Option<usize>,
    ) -> Result<Vec<Todo>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn update_todo(
        &self,
        record_id: &str,
        priority: Option<u8>,
        _status: Option<&str>,
        project: Option<&str>,
        tags: Option<Vec<String>>,
        content: Option<&str>,
    ) -> Result<Todo> {
        let mut store = self.todos.lock().unwrap();
        let todo = store
            .get_mut(record_id)
            .ok_or_else(|| anyhow!("no todo with uuid {record_id}"))?;
        if let Some(priority) = priority {
            todo.priority = priority;
        }
        if let Some(project) = project {
            todo.project = Some(project.to_string());
        }
        if let Some(tags) = tags {
            todo.tags = tags;
        }
        if let Some(content) = content {
            todo.content = content.to_string();
        }
        todo.updated_at = Utc::now();
        Ok(todo.clone())
    }

    async fn delete_todo(&self, _record_id: &str) -> Result<()> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn complete_todo(&self, _record_id: &str) -> Result<()> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn undo_todo(&self, _record_id: &str) -> Result<()> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn search_todos(&self, _query: &str, _project: Option<&str>) -> Result<Vec<Todo>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn get_todo_stats(&self) -> Result<serde_json::Value> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn set_due_date(&self, _record_id: &str, _due_date: Option<&str>) -> Result<()> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn link_deps(
        &self,
        _uuid: &str,
        _blocks: &[String],
        _blocked_by: &[String],
    ) -> Result<()> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn get_todo_by_uuid(&self, uuid: &str) -> Result<Option<Todo>> {
        Ok(self.todos.lock().unwrap().get(uuid).cloned())
    }

    async fn get_todos_by_uuids(&self, _uuids: &[String]) -> Result<Vec<Todo>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn list_all_todos(&self, _project: Option<&str>) -> Result<Vec<Todo>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn list_active_todos(&self) -> Result<Vec<Todo>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn create_notes(
        &self,
        _notes: Vec<(String, Option<String>, Option<String>, Vec<String>)>,
    ) -> Result<Vec<crate::models::Note>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn get_note(&self, _record_id: &str) -> Result<Option<crate::models::Note>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn list_notes(
        &self,
        _project: Option<&str>,
        _limit: Option<usize>,
    ) -> Result<Vec<crate::models::Note>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn delete_note(&self, _record_id: &str) -> Result<()> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn search_notes(
        &self,
        _query: &str,
        _project: Option<&str>,
    ) -> Result<Vec<crate::models::Note>> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }

    async fn execute_raw_query(&self, _query: &str) -> Result<serde_json::Value> {
        unimplemented!("not exercised by DoobCheckpointStore tests")
    }
}
