mod common;

use common::setup_test_db;

#[tokio::test]
async fn test_list_output_formatting() {
    let db = setup_test_db().await;

    // Add todos with various attributes
    doob::commands::add::execute(
        &db,
        vec!["Task 1".to_string()],
        Some(1),
        Some("Project A".to_string()),
        None,
        Some("urgent,backend".to_string()),
    ).await.unwrap();

    doob::commands::add::execute(
        &db,
        vec!["Task 2".to_string()],
        Some(2),
        None,
        None,
        None,
    ).await.unwrap();

    // Get and format todos
    let todos = doob::commands::list::execute(&db, None, None, None).await.unwrap();
    let output = doob::output::format_todos(&todos);

    // Verify output contains expected elements
    assert!(output.contains("Task 1"));
    assert!(output.contains("Task 2"));
    assert!(output.contains("priority: 1"));
    assert!(output.contains("priority: 2"));
    assert!(output.contains("Project: Project A"));
    assert!(output.contains("Tags: urgent, backend"));
    assert!(output.contains("[pending]"));
}

#[tokio::test]
async fn test_list_empty() {
    let db = setup_test_db().await;

    let todos = doob::commands::list::execute(&db, None, None, None).await.unwrap();
    let output = doob::output::format_todos(&todos);

    assert_eq!(output, "No todos found");
}
