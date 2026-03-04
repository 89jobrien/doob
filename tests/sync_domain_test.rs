// tests/sync_domain_test.rs

use doob::sync::domain::{TodoStatus, SyncableTodo, SyncRecord};

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
