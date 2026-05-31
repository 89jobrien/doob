// src/adapters/todo_repository.rs
//
// # TodoRepositoryImpl Adapter
//
// Concrete implementation of the TodoRepository port using SurrealDB.
// This adapter encapsulates all raw SurrealDB query logic.

use crate::db::DbConnection;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use doob_core::ids::{normalize_id, quote_record_id};
use doob_core::models::{Note, Todo, TodoStatus};
use doob_core::ports::TodoRepository;
use doob_core::query_guard::{validate_project, validate_status};
use uuid::Uuid;

const PERCENT: f64 = 100.0;
const ZERO_RATE: f64 = 0.0;

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

    // qual:allow(iosp) reason: "DB adapter — query construction + execution"
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

    // qual:allow(iosp) reason: "DB adapter — query construction + execution"
    async fn list_todos(
        &self,
        status: Option<&str>,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Todo>> {
        let mut query = String::from("SELECT * FROM todo");
        let mut conditions = Vec::new();

        if let Some(s) = status {
            validate_status(s)?;
            conditions.push(format!("status = '{}'", s));
        }

        if let Some(p) = project {
            validate_project(p)?;
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

        let todo = match todos.into_iter().next() {
            Some(t) => t,
            None => return Err(anyhow!("Todo not found: {}", record_id)),
        };

        // Allow undo for completed or cancelled todos
        if todo.status != TodoStatus::Completed && todo.status != TodoStatus::Cancelled {
            return Err(anyhow!(
                "Todo {} is not completed or cancelled (current status: {:?})",
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

    // qual:allow(iosp) reason: "DB adapter — query construction + execution"
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
            completed as f64 / total as f64 * PERCENT
        } else {
            ZERO_RATE
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

    async fn set_due_date(&self, record_id: &str, due_date: Option<&str>) -> Result<()> {
        let record_id = normalize_id(record_id.to_string());

        // Verify the todo exists
        let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
        let mut result = self.db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;
        if todos.is_empty() {
            return Err(anyhow!("Todo not found: {}", record_id));
        }

        let update_query = match due_date {
            Some(date_str) => {
                let parsed = parse_due_date(date_str)?;
                let formatted = parsed.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
                format!(
                    "UPDATE {} SET due_date = <datetime>'{}', updated_at = time::now()",
                    quote_record_id(&record_id),
                    formatted
                )
            }
            None => format!(
                "UPDATE {} SET due_date = NONE, updated_at = time::now()",
                quote_record_id(&record_id)
            ),
        };

        self.db.query(&update_query).await?;
        Ok(())
    }

    async fn link_deps(&self, uuid: &str, blocks: &[String], blocked_by: &[String]) -> Result<()> {
        self.db
            .query("UPDATE todo SET blocks = $blocks, blocked_by = $blocked_by WHERE uuid = $uuid")
            .bind(("uuid", uuid.to_string()))
            .bind(("blocks", blocks.to_vec()))
            .bind(("blocked_by", blocked_by.to_vec()))
            .await?;
        Ok(())
    }

    async fn get_todo_by_uuid(&self, uuid: &str) -> Result<Option<Todo>> {
        let mut result = self
            .db
            .query("SELECT * FROM todo WHERE uuid = $id LIMIT 1")
            .bind(("id", uuid.to_string()))
            .await?;
        let todos: Vec<Todo> = result.take(0)?;
        Ok(todos.into_iter().next())
    }

    async fn get_todos_by_uuids(&self, uuids: &[String]) -> Result<Vec<Todo>> {
        if uuids.is_empty() {
            return Ok(vec![]);
        }
        let mut todos = Vec::new();
        for uuid in uuids {
            let mut result = self
                .db
                .query("SELECT * FROM todo WHERE uuid = $uuid LIMIT 1")
                .bind(("uuid", uuid.clone()))
                .await?;
            let found: Vec<Todo> = result.take(0)?;
            todos.extend(found);
        }
        Ok(todos)
    }

    // qual:allow(iosp) reason: "DB adapter — query construction + execution"
    async fn list_all_todos(&self, project: Option<&str>) -> Result<Vec<Todo>> {
        let mut query = String::from("SELECT * FROM todo");
        if let Some(p) = project {
            validate_project(p)?;
            query.push_str(&format!(" WHERE project = '{}'", p));
        }
        query.push_str(" ORDER BY created_at ASC");
        let mut result = self.db.query(&query).await?;
        Ok(result.take(0)?)
    }

    async fn list_active_todos(&self) -> Result<Vec<Todo>> {
        let mut result = self
            .db
            .query("SELECT * FROM todo WHERE status IN ['pending', 'in_progress']")
            .await?;
        Ok(result.take(0)?)
    }

    // ========================================================================
    // NOTE OPERATIONS
    // ========================================================================

    // qual:allow(iosp) reason: "DB adapter — query construction + execution"
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

    // qual:allow(iosp) reason: "DB adapter — query construction + execution"
    async fn list_notes(&self, project: Option<&str>, limit: Option<usize>) -> Result<Vec<Note>> {
        let mut query = String::from("SELECT * FROM note");

        if let Some(p) = project {
            validate_project(p)?;
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

    // qual:allow(iosp) reason: "DB adapter — query construction + execution"
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

fn parse_due_date(date_str: &str) -> Result<DateTime<Utc>> {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(date.and_time(NaiveTime::MIN).and_utc());
    }
    Err(anyhow!(
        "Invalid date format: '{}'. Expected YYYY-MM-DD",
        date_str
    ))
}
