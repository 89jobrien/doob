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

use crate::models::handoff_item::{ExtraEntry, HandoffItem};
use crate::models::{ArchivedTodo, Note, Todo};
use anyhow::Result;
use async_trait::async_trait;

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

    /// Set or clear a due date on a todo.
    /// Pass `None` to clear, `Some("YYYY-MM-DD")` to set.
    async fn set_due_date(&self, record_id: &str, due_date: Option<&str>) -> Result<()>;

    /// Link dependency UUIDs (blocks/blocked_by) on a todo
    async fn link_deps(&self, uuid: &str, blocks: &[String], blocked_by: &[String]) -> Result<()>;

    /// Fetch a todo by UUID (not record ID)
    async fn get_todo_by_uuid(&self, uuid: &str) -> Result<Option<Todo>>;

    /// Fetch multiple todos by their UUIDs
    async fn get_todos_by_uuids(&self, uuids: &[String]) -> Result<Vec<Todo>>;

    /// List all todos ordered by created_at ASC (for kanban board)
    async fn list_all_todos(&self, project: Option<&str>) -> Result<Vec<Todo>>;

    /// List active (pending + in_progress) todos (for cache building)
    async fn list_active_todos(&self) -> Result<Vec<Todo>>;

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
    async fn list_notes(&self, project: Option<&str>, limit: Option<usize>) -> Result<Vec<Note>>;

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

// ============================================================================
// HANDOFF REPOSITORY PORT
// ============================================================================

/// Abstraction for all handoff item persistence operations.
#[async_trait]
pub trait HandoffRepository: Send + Sync {
    /// Find a handoff item by its handoff_id
    async fn get_by_handoff_id(&self, handoff_id: &str) -> Result<Option<HandoffItem>>;

    /// List handoff items with optional filters
    async fn list_handoff_items(
        &self,
        project: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<HandoffItem>>;

    /// Create a handoff item via raw SQL (datetime injection needed for SurrealDB)
    async fn create_handoff_raw(&self, sql: &str) -> Result<()>;

    /// Update a handoff item via raw SQL
    async fn update_handoff_raw(&self, sql: &str) -> Result<()>;

    /// Update status of a handoff item
    async fn update_handoff_status(&self, handoff_id: &str, status: &str) -> Result<()>;

    /// Append an extra entry to a handoff item
    async fn add_extra(&self, handoff_id: &str, entry: ExtraEntry) -> Result<()>;
}

// ============================================================================
// HANDOFF LOG & STATE REPOSITORY PORT
// ============================================================================

/// Persistence for handoff session logs, state, and handup checkpoints.
/// Split from HandoffRepository to keep the sync-oriented trait lean.
#[async_trait]
pub trait HandoffSessionRepository: Send + Sync {
    /// Append a session log entry
    async fn log_append(
        &self,
        project: &str,
        date: &str,
        summary: &str,
        commits: &[String],
    ) -> Result<()>;

    /// Query log entries for a project (most recent first)
    async fn log_query(&self, project: &str) -> Result<Vec<crate::models::handoff::LogEntry>>;

    /// Save session state for a project (branch, build, tests, etc.)
    async fn save_state(
        &self,
        project: &str,
        state: &crate::models::handoff::HandoffState,
    ) -> Result<()>;

    /// Load session state for a project
    async fn load_state(
        &self,
        project: &str,
    ) -> Result<Option<crate::models::handoff::HandoffState>>;

    /// Save a handup checkpoint
    async fn save_checkpoint(
        &self,
        checkpoint: &crate::models::handoff::HandupCheckpoint,
    ) -> Result<()>;
}

// ============================================================================
// ARCHIVE REPOSITORY PORT
// ============================================================================

/// Abstraction for archive persistence operations.
#[async_trait]
pub trait ArchiveRepository: Send + Sync {
    /// Find archival candidates: completed/cancelled todos older than cutoff
    async fn find_archive_candidates(
        &self,
        cutoff_iso: &str,
        project: Option<&str>,
    ) -> Result<Vec<Todo>>;

    /// Archive a single todo (create archive record + delete original)
    async fn archive_todo(&self, todo: &Todo) -> Result<()>;

    /// List archived todos with optional filters
    async fn list_archived(
        &self,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<ArchivedTodo>>;
}
