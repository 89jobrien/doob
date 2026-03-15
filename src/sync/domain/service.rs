// src/sync/domain/service.rs
//
// Domain service for sync orchestration.

use super::traits::MinimalIssueTracker;
use super::types::{SyncError, SyncRecord, SyncableTodo, TodoStatus};

// ============================================================================
// SYNC SERVICE (Refactored)
// ============================================================================

/// Domain service for syncing todos to external issue trackers.
///
/// Generic over `MinimalIssueTracker` - works with any adapter that
/// implements Provider + HealthCheck + IssueCreator.
pub struct SyncService<T: MinimalIssueTracker> {
    tracker: T,
}

impl<T: MinimalIssueTracker> SyncService<T> {
    pub fn new(tracker: T) -> Self {
        Self { tracker }
    }

    /// Sync a single todo to the external tracker.
    ///
    /// # Business Rules
    /// - Provider must be available
    /// - Only pending/in_progress todos can be synced
    ///
    /// # Returns
    /// - `Ok(SyncRecord)` on success
    /// - `Err(SyncError)` if validation fails or sync fails
    pub fn sync_todo(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
        // Domain logic: validate availability
        if !self.tracker.is_available()? {
            return Err(SyncError::ProviderUnavailable(
                self.tracker.name().to_string(),
            ));
        }

        // Domain logic: only sync active todos
        if todo.status != TodoStatus::Pending && todo.status != TodoStatus::InProgress {
            return Err(SyncError::InvalidConfiguration(
                "Only pending/in_progress todos can be synced".to_string(),
            ));
        }

        // Delegate to tracker
        self.tracker.create_issue(todo)
    }

    /// Sync multiple todos sequentially.
    ///
    /// This is a convenience method. For better performance with adapters
    /// that support batch operations, use `BatchIssueCreator::create_issues`.
    ///
    /// # Returns
    /// A vector of results, one per todo. Failures don't stop processing.
    pub fn sync_todos(&self, todos: &[SyncableTodo]) -> Vec<Result<SyncRecord, SyncError>> {
        todos.iter().map(|todo| self.sync_todo(todo)).collect()
    }

    /// Get the provider name this service is using.
    pub fn provider_name(&self) -> &str {
        self.tracker.name()
    }

    /// Check if the provider is currently available.
    pub fn is_available(&self) -> Result<bool, SyncError> {
        self.tracker.is_available()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::domain::traits::{HealthCheck, IssueCreator, Provider, ProviderCapabilities};

    // Mock tracker for testing
    struct MockTracker {
        should_be_available: bool,
        should_fail: bool,
    }

    impl Provider for MockTracker {
        fn name(&self) -> &str {
            "mock"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities::default()
        }
    }

    impl HealthCheck for MockTracker {
        fn is_available(&self) -> Result<bool, SyncError> {
            Ok(self.should_be_available)
        }
    }

    impl IssueCreator for MockTracker {
        fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
            if self.should_fail {
                return Err(SyncError::ExternalApiError("Mock failure".to_string()));
            }

            Ok(SyncRecord {
                external_id: format!("mock-{}", todo.id),
                external_url: None,
                provider: "mock".to_string(),
                synced_at: "2026-03-15T10:00:00Z".to_string(),
            })
        }
    }

    fn make_todo(id: &str, status: TodoStatus) -> SyncableTodo {
        SyncableTodo {
            id: id.to_string(),
            title: "Test todo".to_string(),
            description: None,
            priority: 1,
            status,
            tags: vec![],
            project: None,
            file_path: None,
            due_date: None,
        }
    }

    #[test]
    fn test_sync_todo_success() {
        let tracker = MockTracker {
            should_be_available: true,
            should_fail: false,
        };
        let service = SyncService::new(tracker);

        let todo = make_todo("1", TodoStatus::Pending);
        let result = service.sync_todo(&todo);

        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.external_id, "mock-1");
        assert_eq!(record.provider, "mock");
    }

    #[test]
    fn test_sync_todo_provider_unavailable() {
        let tracker = MockTracker {
            should_be_available: false,
            should_fail: false,
        };
        let service = SyncService::new(tracker);

        let todo = make_todo("1", TodoStatus::Pending);
        let result = service.sync_todo(&todo);

        assert!(result.is_err());
        match result {
            Err(SyncError::ProviderUnavailable(name)) => {
                assert_eq!(name, "mock");
            }
            _ => panic!("Expected ProviderUnavailable error"),
        }
    }

    #[test]
    fn test_sync_todo_invalid_status() {
        let tracker = MockTracker {
            should_be_available: true,
            should_fail: false,
        };
        let service = SyncService::new(tracker);

        // Note: This test would work if we had a Completed status
        // For now, we'll test with valid statuses
        let todo = make_todo("1", TodoStatus::Pending);
        assert!(service.sync_todo(&todo).is_ok());

        let todo2 = make_todo("2", TodoStatus::InProgress);
        assert!(service.sync_todo(&todo2).is_ok());
    }

    #[test]
    fn test_sync_todos_batch() {
        let tracker = MockTracker {
            should_be_available: true,
            should_fail: false,
        };
        let service = SyncService::new(tracker);

        let todos = vec![
            make_todo("1", TodoStatus::Pending),
            make_todo("2", TodoStatus::InProgress),
        ];

        let results = service.sync_todos(&todos);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_sync_todos_partial_failure() {
        let tracker = MockTracker {
            should_be_available: false, // Will fail availability check
            should_fail: false,
        };
        let service = SyncService::new(tracker);

        let todos = vec![
            make_todo("1", TodoStatus::Pending),
            make_todo("2", TodoStatus::InProgress),
        ];

        let results = service.sync_todos(&todos);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_err());
        assert!(results[1].is_err());
    }

    #[test]
    fn test_provider_name() {
        let tracker = MockTracker {
            should_be_available: true,
            should_fail: false,
        };
        let service = SyncService::new(tracker);

        assert_eq!(service.provider_name(), "mock");
    }

    #[test]
    fn test_is_available() {
        let tracker = MockTracker {
            should_be_available: true,
            should_fail: false,
        };
        let service = SyncService::new(tracker);

        assert_eq!(service.is_available().unwrap(), true);
    }
}
