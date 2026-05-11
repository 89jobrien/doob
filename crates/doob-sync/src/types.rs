// src/sync/domain/types.rs
//
// # Domain Types for Sync Operations
//
// This module contains the core domain models (value objects) used throughout
// the sync module. These types are framework-agnostic and have no external
// dependencies.
//
// ## Types
//
// ### Value Objects
//
// - **`SyncableTodo`** - Represents a todo prepared for sync
//   - Contains all fields needed by external trackers
//   - Derived from doob's internal `Todo` model
//
// - **`SyncRecord`** - Metadata about a completed sync
//   - External issue ID and URL
//   - Provider name and timestamp
//   - Stored in local database for tracking
//
// - **`TodoStatus`** - Simple status enum
//   - `Pending` or `InProgress`
//   - Completed todos are not synced
//
// ### Error Type
//
// - **`SyncError`** - All sync-related errors
//   - Uses `thiserror` for ergonomic error handling
//   - Cloneable for batch operations
//
// ## Design Notes
//
// All types are:
// - Serializable (derive `Serialize`, `Deserialize`)
// - Clonable (for batch operations)
// - Framework-agnostic (no axum, tokio, etc.)
// - Pure data structures (no behavior)

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Error, Debug, Clone)]
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

// ============================================================================
// DOMAIN TYPES
// ============================================================================

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
