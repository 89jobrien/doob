// src/ports.rs
//
// # TodoRepository Port (Hexagonal Architecture)
//
// This module defines the `TodoRepository` port, which abstracts all database
// operations for todos and notes. This mirrors the hexagonal pattern already used
// in `src/sync/` and allows commands to depend on the interface rather than the
// concrete database implementation.
//
// ## Architecture
//
// ```text
// Application (commands/)
//     ↓
// Domain Layer (ports.rs)
//   - TodoRepository (interface)
//   - Note operations interface
//     ↓
// Adapter Layer (adapters/todo_repository.rs)
//   - TodoRepositoryImpl (SurrealDB implementation)
// ```
//
// ## Design Principles
//
// 1. **Dependency Inversion** - Commands depend on `TodoRepository` trait, not concrete DB
// 2. **Single Responsibility** - TodoRepository handles all todo/note persistence
// 3. **Testability** - Enables mock implementations for testing

use async_trait::async_trait;
use crate::models::{Note, Todo};
use anyhow::Result;

/// TodoRepository port: Abstraction for all todo and note persistence operations.
///
/// This trait defines all database operations needed by commands. Implementations
/// (adapters) provide concrete implementations using SurrealDB or other backends.
#[async_trait]
pub trait TodoRepository: Send + Sync {
    // ========================================================================
    // TODO OPERATIONS
    // ========================================================================

    /// Create one or more todos
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
    ) -> Result<Vec<Todo>>;

    /// Retrieve a single todo by ID
    async fn get_todo(&self, record_id: &str) -> Result<Option<Todo>>;

    /// List todos with optional filtering
    async fn list_todos(
        &self,
        status: Option<&str>,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Todo>>;

    /// Update a todo's fields
    async fn update_todo(
        &self,
        record_id: &str,
        priority: Option<u8>,
        status: Option<&str>,
        project: Option<&str>,
        tags: Option<Vec<String>>,
        content: Option<&str>,
    ) -> Result<Todo>;

    /// Delete a todo by ID
    async fn delete_todo(&self, record_id: &str) -> Result<()>;

    /// Mark a todo as completed
    async fn complete_todo(&self, record_id: &str) -> Result<()>;

    /// Revert a completed todo back to pending
    async fn undo_todo(&self, record_id: &str) -> Result<()>;

    /// Search todos by content and optional project filter
    async fn search_todos(&self, query: &str, project: Option<&str>) -> Result<Vec<Todo>>;

    /// Get stats for todos (count by status, etc.)
    async fn get_todo_stats(&self) -> Result<serde_json::Value>;

    // ========================================================================
    // NOTE OPERATIONS
    // ========================================================================

    /// Create one or more notes
    async fn create_notes(
        &self,
        notes: Vec<(String, Option<String>, Option<String>, Vec<String>)>,
    ) -> Result<Vec<Note>>;

    /// Retrieve a single note by ID
    async fn get_note(&self, record_id: &str) -> Result<Option<Note>>;

    /// List notes with optional filtering
    async fn list_notes(
        &self,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Note>>;

    /// Delete a note by ID
    async fn delete_note(&self, record_id: &str) -> Result<()>;

    /// Search notes by content and optional project filter
    async fn search_notes(&self, query: &str, project: Option<&str>) -> Result<Vec<Note>>;

    // ========================================================================
    // BATCH OPERATIONS
    // ========================================================================

    /// Execute raw SurrealDB query for special cases
    async fn execute_raw_query(&self, query: &str) -> Result<serde_json::Value>;
}
