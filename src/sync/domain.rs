// src/sync/domain.rs

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Provider '{0}' is not available or not installed")]
    ProviderUnavailable(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("External API error: {0}")]
    ExternalApiError(String),

    #[error("Todo '{0}' has already been synced to this provider")]
    TodoAlreadySynced(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncableTodo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: u8,
    pub status: TodoStatus,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub file_path: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub external_id: String,
    pub external_url: Option<String>,
    pub provider: String,
    pub synced_at: String,
}

/// Port: External issue tracker
pub trait IssueTracker {
    /// Provider name (e.g., "beads", "github")
    fn name(&self) -> &str;

    /// Check if provider is available (CLI installed, auth configured, etc.)
    fn is_available(&self) -> Result<bool, SyncError>;

    /// Create an issue in the external system
    fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError>;
}

/// Domain service: Sync orchestration
pub struct SyncService<T: IssueTracker> {
    tracker: T,
}

impl<T: IssueTracker> SyncService<T> {
    pub fn new(tracker: T) -> Self {
        Self { tracker }
    }

    pub fn sync_todo(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
        // Domain logic: validate availability
        if !self.tracker.is_available()? {
            return Err(SyncError::ProviderUnavailable(
                self.tracker.name().to_string()
            ));
        }

        // Domain logic: only sync active todos
        if todo.status != TodoStatus::Pending && todo.status != TodoStatus::InProgress {
            return Err(SyncError::InvalidConfiguration(
                "Only pending/in_progress todos can be synced".to_string()
            ));
        }

        self.tracker.create_issue(todo)
    }

    pub fn sync_todos(&self, todos: &[SyncableTodo]) -> Vec<Result<SyncRecord, SyncError>> {
        todos.iter()
            .map(|todo| self.sync_todo(todo))
            .collect()
    }
}
