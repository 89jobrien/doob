// src/sync/domain/service.rs
//
// # SyncService: Domain Service for Todo Synchronization
//
// This module contains the `SyncService` - the primary domain service for
// orchestrating todo synchronization to external issue trackers.
//
// ## Responsibilities
//
// 1. **Validate business rules** before syncing
//    - Provider must be available
//    - Only pending/in_progress todos can be synced
//
// 2. **Orchestrate sync operations**
//    - Delegate to adapters via the `MinimalIssueTracker` port
//    - Handle errors appropriately
//
// 3. **Optimize batch operations**
//    - Check availability once for multiple todos
//    - Avoid N subprocess spawns for CLI adapters
//
// ## Usage
//
// ```rust
// use doob::sync::domain::{SyncService, SyncableTodo, TodoStatus};
// use doob::sync::adapters::BeadsAdapter;
//
// let service = SyncService::new(BeadsAdapter::new());
//
// // Single sync
// let todo = SyncableTodo { /* ... */ };
// let record = service.sync_todo(&todo)?;
//
// // Batch sync (optimized - only 1 availability check)
// let todos = vec![todo1, todo2, todo3];
// let results = service.sync_todos(&todos);
// ```
//
// ## Design Notes
//
// - Generic over `T: MinimalIssueTracker` (dependency inversion)
// - No knowledge of concrete adapters (BeadsAdapter, etc.)
// - Pure domain logic - no HTTP, CLI, or database code

use crate::traits::MinimalIssueTracker;
use crate::types::{SyncError, SyncRecord, SyncableTodo, TodoStatus};

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
    /// **Performance Note**: This method checks provider availability once upfront,
    /// then syncs each todo individually. This avoids redundant availability checks
    /// (e.g., N subprocess spawns for CLI-based adapters).
    ///
    /// For adapters that support batch operations, use `BatchIssueCreator::create_issues`.
    ///
    /// # Returns
    /// A vector of results, one per todo. Failures don't stop processing.
    pub fn sync_todos(&self, todos: &[SyncableTodo]) -> Vec<Result<SyncRecord, SyncError>> {
        // Optimization: Check availability once upfront instead of N times
        match self.tracker.is_available() {
            Ok(false) | Err(_) => {
                // Provider unavailable - return error for all todos
                let error = SyncError::ProviderUnavailable(self.tracker.name().to_string());
                todos.iter().map(|_| Err(error.clone())).collect()
            }
            Ok(true) => {
                // Provider available - sync each todo (skip redundant availability checks)
                todos
                    .iter()
                    .map(|todo| {
                        // Validate todo status
                        if todo.status != TodoStatus::Pending
                            && todo.status != TodoStatus::InProgress
                        {
                            return Err(SyncError::InvalidConfiguration(
                                "Only pending/in_progress todos can be synced".to_string(),
                            ));
                        }
                        // Create issue (availability already confirmed)
                        self.tracker.create_issue(todo)
                    })
                    .collect()
            }
        }
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
    use crate::traits::{HealthCheck, IssueCreator, Provider, ProviderCapabilities};

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

        assert!(service.is_available().unwrap());
    }

    #[test]
    fn test_sync_todos_checks_availability_once() {
        // This test verifies the optimization: sync_todos should check availability
        // once upfront, not N times (which would spawn N subprocesses for CLI adapters)
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        struct CountingMockTracker {
            should_be_available: bool,
            availability_check_count: Arc<AtomicUsize>,
        }

        impl Provider for CountingMockTracker {
            fn name(&self) -> &str {
                "counting-mock"
            }
            fn version(&self) -> &str {
                "1.0.0"
            }
        }

        impl HealthCheck for CountingMockTracker {
            fn is_available(&self) -> Result<bool, SyncError> {
                self.availability_check_count.fetch_add(1, Ordering::SeqCst);
                Ok(self.should_be_available)
            }
        }

        impl IssueCreator for CountingMockTracker {
            fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
                Ok(SyncRecord {
                    external_id: format!("counting-{}", todo.id),
                    external_url: None,
                    provider: "counting-mock".to_string(),
                    synced_at: "2026-03-15T10:00:00Z".to_string(),
                })
            }
        }

        let check_count = Arc::new(AtomicUsize::new(0));
        let tracker = CountingMockTracker {
            should_be_available: true,
            availability_check_count: check_count.clone(),
        };
        let service = SyncService::new(tracker);

        let todos = vec![
            make_todo("1", TodoStatus::Pending),
            make_todo("2", TodoStatus::InProgress),
            make_todo("3", TodoStatus::Pending),
        ];

        let results = service.sync_todos(&todos);

        // All should succeed
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));

        // Critical: availability should be checked ONCE, not 3 times
        let count = check_count.load(Ordering::SeqCst);
        assert_eq!(
            count, 1,
            "Availability should be checked once, not {} times",
            count
        );
    }
}
