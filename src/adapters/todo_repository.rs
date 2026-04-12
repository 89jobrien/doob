// src/adapters/todo_repository.rs
//
// # TodoRepositoryImpl Adapter
//
// Concrete implementation of the TodoRepository port using SurrealDB.
// This adapter encapsulates all raw SurrealDB query logic.

use async_trait::async_trait;
use crate::commands::{normalize_id, quote_record_id};
use crate::db::DbConnection;
use crate::models::{Note, Todo, TodoStatus};
use crate::ports::TodoRepository;
use anyhow::{anyhow, Result};
use uuid::Uuid;

/// SurrealDB-backed implementation of TodoRepository
pub struct TodoRepositoryImpl {
    db: DbConnection,
}

impl TodoRepositoryImpl {
    /// Create a new repository instance with the given database connection
    pub fn new(db: DbConnection) -> Self {
        TodoRepositoryImpl { db }
    }
}

#[async_trait]
impl TodoRepository for TodoRepositoryImpl {
    // ========================================================================
    // TODO OPERATIONS
    // ========================================================================

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
        let mut created_todos = Vec::new();

        for (uuid, content, priority, project, file_path, tags) in todos {
            let mut query =
                String::from("CREATE todo SET uuid = $uuid, content = $content, status = 'pending', priority = $priority, tags = $tags");

            if project.is_some() {
                query.push_str(", project = $project");
            }

            if file_path.is_some() {
                query.push_str(", file_path = $file_path");
            }

            let mut query_builder = self
                .db
                .query(&query)
                .bind(("uuid", uuid))
                .bind(("content", content))
                .bind(("priority", priority))
                .bind(("tags", tags.clone()));

            if let Some(ref proj) = project {
                query_builder = query_builder.bind(("project", proj.clone()));
            }

            if let Some(ref fp) = file_path {
                query_builder = query_builder.bind(("file_path", fp.clone()));
            }

            let mut result = query_builder.await?;
            let created: Option<Todo> = result.take(0)?;

            if let Some(todo) = created {
                created_todos.push(todo);
            }
        }

        Ok(created_todos)
    }

    async fn get_todo(&self, record_id: &str) -> Result<Option<Todo>> {
        let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(record_id));
        let mut result = self.db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;

        Ok(todos.into_iter().next())
    }

    async fn list_todos(
        &self,
        status: Option<&str>,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Todo>> {
        let mut query = String::from("SELECT * FROM todo");
        let mut conditions = Vec::new();

        if let Some(s) = status {
            conditions.push(format!("status = '{}'", s));
        }

        if let Some(p) = project {
            conditions.push(format!("project = '{}'", p));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(l) = limit {
            query.push_str(&format!(" LIMIT {}", l));
        }

        let mut result = self.db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;

        Ok(todos)
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
        let record_id = normalize_id(record_id.to_string());

        // Verify the todo exists
        let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
        let mut result = self.db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;

        if todos.is_empty() {
            return Err(anyhow!("Todo not found: {}", record_id));
        }

        // Build SET clause from provided fields
        let mut set_parts: Vec<String> = vec!["updated_at = time::now()".to_string()];

        if let Some(p) = priority {
            set_parts.push(format!("priority = {}", p));
        }

        if let Some(s) = status {
            set_parts.push(format!("status = '{}'", s));
            if s == "completed" {
                set_parts.push("completed_at = time::now()".to_string());
            }
        }

        if let Some(p) = project {
            set_parts.push(format!("project = '{}'", p.replace('\'', "\\'")));
        }

        if let Some(t) = tags {
            let tag_list: Vec<String> = t
                .iter()
                .map(|s| format!("'{}'", s.replace('\'', "\\'")))
                .collect();
            set_parts.push(format!("tags = [{}]", tag_list.join(", ")));
        }

        if let Some(c) = content {
            set_parts.push(format!("content = '{}'", c.replace('\'', "\\'")));
        }

        let update_query = format!(
            "UPDATE {} SET {}",
            quote_record_id(&record_id),
            set_parts.join(", ")
        );
        self.db.query(&update_query).await?;

        // Fetch and return the updated todo
        let fetch_query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
        let mut fetch_result = self.db.query(&fetch_query).await?;
        let updated: Vec<Todo> = fetch_result.take(0)?;

        updated
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Failed to fetch updated todo: {}", record_id))
    }

    async fn delete_todo(&self, record_id: &str) -> Result<()> {
        let record_id = normalize_id(record_id.to_string());

        // Verify the todo exists before deleting
        let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
        let mut result = self.db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;

        if todos.is_empty() {
            return Err(anyhow!("Todo not found: {}", record_id));
        }

        // Delete the todo
        let delete_query = format!("DELETE {}", quote_record_id(&record_id));
        self.db.query(&delete_query).await?;

        Ok(())
    }

    async fn complete_todo(&self, record_id: &str) -> Result<()> {
        let record_id = normalize_id(record_id.to_string());

        // Get existing todo using query
        let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
        let mut result = self.db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;

        if todos.is_empty() {
            return Err(anyhow!("Todo not found: {}", record_id));
        }

        // Update using query with explicit values
        let update_query = format!(
            "UPDATE {} SET status = 'completed', completed_at = time::now(), updated_at = time::now()",
            quote_record_id(&record_id)
        );
        self.db.query(&update_query).await?;

        Ok(())
    }

    async fn undo_todo(&self, record_id: &str) -> Result<()> {
        let record_id = normalize_id(record_id.to_string());

        // Get existing todo using query
        let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
        let mut result = self.db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;

        if todos.is_empty() {
            return Err(anyhow!("Todo not found: {}", record_id));
        }

        let todo = todos.into_iter().next().unwrap();

        // Only allow undo for completed todos
        if todo.status != TodoStatus::Completed {
            return Err(anyhow!(
                "Todo {} is not completed (current status: {:?})",
                record_id,
                todo.status
            ));
        }

        // Update status back to pending
        let update_query = format!(
            "UPDATE {} SET status = 'pending', updated_at = time::now()",
            quote_record_id(&record_id)
        );
        self.db.query(&update_query).await?;

        Ok(())
    }

    async fn search_todos(&self, query: &str, project: Option<&str>) -> Result<Vec<Todo>> {
        let q = query.to_lowercase();

        let mut query_str = String::from(
            "SELECT * FROM todo WHERE string::contains(string::lowercase(content), $query)",
        );
        if project.is_some() {
            query_str.push_str(" AND project = $project");
        }
        query_str.push_str(" ORDER BY created_at DESC");

        let mut builder = self.db.query(&query_str).bind(("query", q));
        if let Some(p) = project {
            builder = builder.bind(("project", p.to_string()));
        }

        let mut result = builder.await?;
        Ok(result.take(0)?)
    }

    async fn get_todo_stats(&self) -> Result<serde_json::Value> {
        let mut query_res = self.db.query("SELECT * FROM todo").await?;
        let todos: Vec<Todo> = query_res.take(0)?;

        let mut pending = 0usize;
        let mut in_progress = 0usize;
        let mut completed = 0usize;
        let mut cancelled = 0usize;

        for todo in &todos {
            match todo.status {
                TodoStatus::Pending => pending += 1,
                TodoStatus::InProgress => in_progress += 1,
                TodoStatus::Completed => completed += 1,
                TodoStatus::Cancelled => cancelled += 1,
            }
        }

        let total = todos.len();
        let completion_rate = if total > 0 {
            completed as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        let stats = serde_json::json!({
            "total": total,
            "pending": pending,
            "in_progress": in_progress,
            "completed": completed,
            "cancelled": cancelled,
            "completion_rate": completion_rate,
        });

        Ok(stats)
    }

    // ========================================================================
    // NOTE OPERATIONS
    // ========================================================================

    async fn create_notes(
        &self,
        notes: Vec<(String, Option<String>, Option<String>, Vec<String>)>,
    ) -> Result<Vec<Note>> {
        let mut created_notes = Vec::new();

        for (content, project, file_path, tags) in notes {
            let uuid = Uuid::new_v4().to_string();

            let mut query =
                String::from("CREATE note SET uuid = $uuid, content = $content, tags = $tags");

            if project.is_some() {
                query.push_str(", project = $project");
            }

            if file_path.is_some() {
                query.push_str(", file_path = $file_path");
            }

            let mut qb = self
                .db
                .query(&query)
                .bind(("uuid", uuid))
                .bind(("content", content))
                .bind(("tags", tags.clone()));

            if let Some(ref proj) = project {
                qb = qb.bind(("project", proj.clone()));
            }

            if let Some(ref fp) = file_path {
                qb = qb.bind(("file_path", fp.clone()));
            }

            let mut result = qb.await?;
            let created: Option<Note> = result.take(0)?;

            if let Some(note) = created {
                created_notes.push(note);
            }
        }

        Ok(created_notes)
    }

    async fn get_note(&self, record_id: &str) -> Result<Option<Note>> {
        let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(record_id));
        let mut result = self.db.query(&query).await?;
        let notes: Vec<Note> = result.take(0)?;

        Ok(notes.into_iter().next())
    }

    async fn list_notes(
        &self,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Note>> {
        let mut query = String::from("SELECT * FROM note");

        if let Some(p) = project {
            query.push_str(&format!(" WHERE project = '{}'", p));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(l) = limit {
            query.push_str(&format!(" LIMIT {}", l));
        }

        let mut result = self.db.query(&query).await?;
        let notes: Vec<Note> = result.take(0)?;

        Ok(notes)
    }

    async fn delete_note(&self, record_id: &str) -> Result<()> {
        let record_id = normalize_id(record_id.to_string());

        // Verify the note exists before deleting
        let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
        let mut result = self.db.query(&query).await?;
        let notes: Vec<Note> = result.take(0)?;

        if notes.is_empty() {
            return Err(anyhow!("Note not found: {}", record_id));
        }

        // Delete the note
        let delete_query = format!("DELETE {}", quote_record_id(&record_id));
        self.db.query(&delete_query).await?;

        Ok(())
    }

    async fn search_notes(&self, query: &str, project: Option<&str>) -> Result<Vec<Note>> {
        let q = query.to_lowercase();

        let mut query_str = String::from(
            "SELECT * FROM note WHERE string::contains(string::lowercase(content), $query)",
        );
        if project.is_some() {
            query_str.push_str(" AND project = $project");
        }
        query_str.push_str(" ORDER BY created_at DESC");

        let mut builder = self.db.query(&query_str).bind(("query", q));
        if let Some(p) = project {
            builder = builder.bind(("project", p.to_string()));
        }

        let mut result = builder.await?;
        Ok(result.take(0)?)
    }

    // ========================================================================
    // BATCH OPERATIONS
    // ========================================================================

    async fn execute_raw_query(&self, query: &str) -> Result<serde_json::Value> {
        let mut result = self.db.query(query).await?;
        let values: Vec<serde_json::Value> = result.take(0)?;
        Ok(serde_json::json!(values))
    }
}
