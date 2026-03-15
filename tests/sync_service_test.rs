// tests/sync_service_test.rs

mod common;

use common::sync_mocks::{make_test_todo, MockMinimalTracker};
use doob::sync::domain::{SyncError, SyncService, TodoStatus};

#[test]
fn sync_service__creates_issue__when_todo_is_pending() {
    let tracker = MockMinimalTracker::new("test");
    let service = SyncService::new(tracker);
    let todo = make_test_todo("1", "Test todo", TodoStatus::Pending);

    let result = service.sync_todo(&todo);

    assert!(result.is_ok());
    let record = result.unwrap();
    assert_eq!(record.external_id, "test-1");
}

#[test]
fn sync_service__creates_issue__when_todo_is_in_progress() {
    let tracker = MockMinimalTracker::new("test");
    let service = SyncService::new(tracker);
    let todo = make_test_todo("2", "Test todo", TodoStatus::InProgress);

    let result = service.sync_todo(&todo);

    assert!(result.is_ok());
}

#[test]
fn sync_service__rejects__when_provider_unavailable() {
    let tracker = MockMinimalTracker::new("test").with_availability(false);
    let service = SyncService::new(tracker);
    let todo = make_test_todo("1", "Test todo", TodoStatus::Pending);

    let result = service.sync_todo(&todo);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SyncError::ProviderUnavailable(_)));
}

#[test]
fn sync_service__handles_multiple_todos() {
    let tracker = MockMinimalTracker::new("test");
    let service = SyncService::new(tracker);

    let todos = vec![
        make_test_todo("1", "Todo 1", TodoStatus::Pending),
        make_test_todo("2", "Todo 2", TodoStatus::InProgress),
        make_test_todo("3", "Todo 3", TodoStatus::Pending),
    ];

    let results = service.sync_todos(&todos);

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[test]
fn sync_service__partial_failure_continues() {
    let tracker = MockMinimalTracker::new("test").with_failure(true);
    let service = SyncService::new(tracker);

    let todos = vec![
        make_test_todo("1", "Todo 1", TodoStatus::Pending),
        make_test_todo("2", "Todo 2", TodoStatus::Pending),
    ];

    let results = service.sync_todos(&todos);

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_err()));
}
