// tests/common/sync_mocks.rs
//
// Shared mock implementations for sync module testing.

use doob::sync::domain::{
    HealthCheck, IssueCreator, Provider, ProviderCapabilities, SyncError, SyncRecord, SyncableTodo,
};

/// Mock tracker for testing MinimalIssueTracker implementations.
///
/// Configurable via builder pattern for different test scenarios.
pub struct MockMinimalTracker {
    name: String,
    available: bool,
    should_fail: bool,
}

impl MockMinimalTracker {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            available: true,
            should_fail: false,
        }
    }

    pub fn with_availability(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

impl Provider for MockMinimalTracker {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0-test"
    }

    // Uses default implementation (all false)
}

impl HealthCheck for MockMinimalTracker {
    fn is_available(&self) -> Result<bool, SyncError> {
        Ok(self.available)
    }
}

impl IssueCreator for MockMinimalTracker {
    fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
        if self.should_fail {
            return Err(SyncError::ExternalApiError("Mock failure".to_string()));
        }

        if !self.available {
            return Err(SyncError::ProviderUnavailable(self.name.clone()));
        }

        Ok(SyncRecord {
            external_id: format!("{}-{}", self.name, todo.id),
            external_url: None,
            provider: self.name.clone(),
            synced_at: "2026-03-15T10:00:00Z".to_string(),
        })
    }
}

/// Create a test todo with configurable fields.
pub fn make_test_todo(
    id: &str,
    title: &str,
    status: doob::sync::domain::TodoStatus,
) -> SyncableTodo {
    SyncableTodo {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        priority: 2,
        status,
        tags: vec![],
        project: None,
        file_path: None,
        due_date: None,
    }
}

/// Create a simple test todo with default title.
pub fn make_simple_todo(id: &str, status: doob::sync::domain::TodoStatus) -> SyncableTodo {
    make_test_todo(id, "Test todo", status)
}
