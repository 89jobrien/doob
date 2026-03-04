// tests/sync_service_test.rs

use doob::sync::domain::{IssueTracker, SyncableTodo, SyncRecord, SyncError, TodoStatus, SyncService};

// Reuse MockIssueTracker from domain_test or define it here
struct MockTracker {
    name: String,
    available: bool,
    should_fail: bool,
}

impl MockTracker {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            available: true,
            should_fail: false,
        }
    }

    fn with_availability(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

impl IssueTracker for MockTracker {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> Result<bool, SyncError> {
        Ok(self.available)
    }

    fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
        if self.should_fail {
            return Err(SyncError::ExternalApiError("Mock failure".to_string()));
        }

        Ok(SyncRecord {
            external_id: format!("{}-{}", self.name, todo.id),
            external_url: None,
            provider: self.name.clone(),
            synced_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

fn make_todo(id: &str, title: &str, status: TodoStatus) -> SyncableTodo {
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

#[test]
fn sync_service__creates_issue__when_todo_is_pending() {
    let tracker = MockTracker::new("test");
    let service = SyncService::new(tracker);
    let todo = make_todo("1", "Test todo", TodoStatus::Pending);

    let result = service.sync_todo(&todo);

    assert!(result.is_ok());
    let record = result.unwrap();
    assert_eq!(record.external_id, "test-1");
}

#[test]
fn sync_service__creates_issue__when_todo_is_in_progress() {
    let tracker = MockTracker::new("test");
    let service = SyncService::new(tracker);
    let todo = make_todo("2", "Test todo", TodoStatus::InProgress);

    let result = service.sync_todo(&todo);

    assert!(result.is_ok());
}

#[test]
fn sync_service__rejects__when_provider_unavailable() {
    let tracker = MockTracker::new("test").with_availability(false);
    let service = SyncService::new(tracker);
    let todo = make_todo("1", "Test todo", TodoStatus::Pending);

    let result = service.sync_todo(&todo);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SyncError::ProviderUnavailable(_)));
}

#[test]
fn sync_service__handles_multiple_todos() {
    let tracker = MockTracker::new("test");
    let service = SyncService::new(tracker);

    let todos = vec![
        make_todo("1", "Todo 1", TodoStatus::Pending),
        make_todo("2", "Todo 2", TodoStatus::InProgress),
        make_todo("3", "Todo 3", TodoStatus::Pending),
    ];

    let results = service.sync_todos(&todos);

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[test]
fn sync_service__partial_failure_continues() {
    let tracker = MockTracker::new("test").with_failure(true);
    let service = SyncService::new(tracker);

    let todos = vec![
        make_todo("1", "Todo 1", TodoStatus::Pending),
        make_todo("2", "Todo 2", TodoStatus::Pending),
    ];

    let results = service.sync_todos(&todos);

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_err()));
}
