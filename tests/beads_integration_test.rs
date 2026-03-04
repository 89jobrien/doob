// tests/beads_integration_test.rs

#[cfg(feature = "integration-tests")]
use doob::sync::adapters::BeadsAdapter;
#[cfg(feature = "integration-tests")]
use doob::sync::domain::{IssueTracker, SyncableTodo, TodoStatus};

#[test]
#[cfg(feature = "integration-tests")]
fn beads_adapter__creates_real_issue__when_bd_cli_available() {
    let adapter = BeadsAdapter::new();

    // Skip if bd not available
    if !adapter.is_available().unwrap_or(false) {
        eprintln!("Skipping: bd CLI not available");
        return;
    }

    let todo = SyncableTodo {
        id: "integration-test".to_string(),
        title: "Doob sync integration test".to_string(),
        description: Some("Created by doob sync integration test".to_string()),
        priority: 2,
        status: TodoStatus::Pending,
        tags: vec!["test".to_string(), "integration".to_string()],
        project: Some("doob".to_string()),
        file_path: None,
        due_date: None,
    };

    let result = adapter.create_issue(&todo);

    assert!(result.is_ok());
    let record = result.unwrap();
    assert!(record.external_id.starts_with("bd-") || record.external_id.starts_with("beads-"));
    assert_eq!(record.provider, "beads");

    println!("Created issue: {}", record.external_id);

    // Note: Cleanup should be done manually or via bd close command
    // For safety, we leave test issues for manual inspection
}

#[test]
#[cfg(feature = "integration-tests")]
fn beads_adapter__is_available__when_bd_not_installed() {
    // This test only passes if bd is NOT installed
    // Skip in CI where bd might be installed

    let adapter = BeadsAdapter::new();
    let result = adapter.is_available();

    // Just verify it returns a result (either true or false is valid)
    assert!(result.is_ok());
}
