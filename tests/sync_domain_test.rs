// tests/sync_domain_test.rs

use doob::sync::domain::{TodoStatus, SyncableTodo, SyncRecord, SyncError, IssueTracker};

// Mock implementation for testing IssueTracker trait
struct MockIssueTracker {
    name: String,
    available: bool,
}

impl MockIssueTracker {
    fn new(name: &str, available: bool) -> Self {
        Self {
            name: name.to_string(),
            available,
        }
    }
}

impl doob::sync::domain::IssueTracker for MockIssueTracker {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> Result<bool, SyncError> {
        Ok(self.available)
    }

    fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
        if !self.available {
            return Err(SyncError::ProviderUnavailable(self.name.clone()));
        }

        Ok(SyncRecord {
            external_id: format!("{}-{}", self.name, todo.id),
            external_url: None,
            provider: self.name.clone(),
            synced_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[test]
fn todo_status__serializes_to_string() {
    let status = TodoStatus::Pending;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"Pending\"");
}

#[test]
fn todo_status__deserializes_from_string() {
    let json = "\"InProgress\"";
    let status: TodoStatus = serde_json::from_str(json).unwrap();
    assert_eq!(status, TodoStatus::InProgress);
}

#[test]
fn todo_status__supports_equality() {
    assert_eq!(TodoStatus::Pending, TodoStatus::Pending);
    assert_ne!(TodoStatus::Pending, TodoStatus::InProgress);
}

#[test]
fn syncable_todo__creates_with_required_fields() {
    let todo = SyncableTodo {
        id: "1".to_string(),
        title: "Test todo".to_string(),
        description: None,
        priority: 2,
        status: TodoStatus::Pending,
        tags: vec![],
        project: None,
        file_path: None,
        due_date: None,
    };

    assert_eq!(todo.id, "1");
    assert_eq!(todo.title, "Test todo");
    assert_eq!(todo.priority, 2);
}

#[test]
fn syncable_todo__serializes_to_json() {
    let todo = SyncableTodo {
        id: "1".to_string(),
        title: "Test".to_string(),
        description: Some("Description".to_string()),
        priority: 1,
        status: TodoStatus::InProgress,
        tags: vec!["tag1".to_string()],
        project: Some("project".to_string()),
        file_path: Some("/path/to/file".to_string()),
        due_date: Some("2026-12-31".to_string()),
    };

    let json = serde_json::to_value(&todo).unwrap();
    assert_eq!(json["id"], "1");
    assert_eq!(json["title"], "Test");
    assert_eq!(json["priority"], 1);
}

#[test]
fn sync_record__stores_external_sync_data() {
    let record = SyncRecord {
        external_id: "bd-42".to_string(),
        external_url: Some("https://example.com/bd-42".to_string()),
        provider: "beads".to_string(),
        synced_at: "2026-03-04T08:00:00Z".to_string(),
    };

    assert_eq!(record.external_id, "bd-42");
    assert_eq!(record.provider, "beads");
    assert!(record.external_url.is_some());
}

#[test]
fn sync_record__serializes_with_optional_url() {
    let record = SyncRecord {
        external_id: "123".to_string(),
        external_url: None,
        provider: "github".to_string(),
        synced_at: "2026-03-04T08:00:00Z".to_string(),
    };

    let json = serde_json::to_value(&record).unwrap();
    assert_eq!(json["external_id"], "123");
    assert_eq!(json["external_url"], serde_json::Value::Null);
}

#[test]
fn sync_error__formats_provider_unavailable() {
    let err = SyncError::ProviderUnavailable("beads".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("beads"));
    assert!(msg.contains("not available"));
}

#[test]
fn sync_error__formats_external_api_error() {
    let err = SyncError::ExternalApiError("Connection timeout".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Connection timeout"));
}

#[test]
fn sync_error__is_debug_formattable() {
    let err = SyncError::InvalidConfiguration("Bad config".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("InvalidConfiguration"));
}

#[test]
fn issue_tracker__returns_provider_name() {
    let tracker = MockIssueTracker::new("test", true);
    assert_eq!(tracker.name(), "test");
}

#[test]
fn issue_tracker__checks_availability() {
    let tracker = MockIssueTracker::new("test", true);
    assert!(tracker.is_available().unwrap());

    let tracker = MockIssueTracker::new("test", false);
    assert!(!tracker.is_available().unwrap());
}

#[test]
fn issue_tracker__creates_issue_when_available() {
    let tracker = MockIssueTracker::new("test", true);
    let todo = SyncableTodo {
        id: "1".to_string(),
        title: "Test".to_string(),
        description: None,
        priority: 2,
        status: TodoStatus::Pending,
        tags: vec![],
        project: None,
        file_path: None,
        due_date: None,
    };

    let result = tracker.create_issue(&todo);
    assert!(result.is_ok());
    let record = result.unwrap();
    assert_eq!(record.external_id, "test-1");
    assert_eq!(record.provider, "test");
}
