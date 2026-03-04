// tests/sync_domain_test.rs

use doob::sync::domain::TodoStatus;

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
