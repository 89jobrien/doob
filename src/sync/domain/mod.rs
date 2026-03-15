// src/sync/domain/mod.rs
//
// Domain layer for sync operations.
//
// This module contains pure business logic with no external dependencies.
// It defines traits (ports), domain types, and domain services.

pub mod traits;
pub mod types;
pub mod service;

// Re-export commonly used items
pub use traits::{
    BatchIssueCreator, ExternalIssueReader, HealthCheck, IssueCreator, IssueDeleter,
    IssueUpdater, Provider, ProviderCapabilities, ProviderHealth,
};

pub use traits::{
    BatchIssueTracker, FullIssueTracker, MinimalIssueTracker, StandardIssueTracker,
};

pub use types::{SyncError, SyncRecord, SyncableTodo, TodoStatus};

pub use service::SyncService;

// Backward compatibility: re-export deprecated IssueTracker
#[allow(deprecated)]
pub use traits::IssueTracker;
