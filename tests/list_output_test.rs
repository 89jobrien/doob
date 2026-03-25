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
    )
    .await
    .unwrap();

    doob::commands::add::execute(&db, vec!["Task 2".to_string()], Some(2), None, None, None)
        .await
        .unwrap();

    // Get and format todos
    let todos = doob::commands::list::execute(&db, None, None, None)
        .await
        .unwrap();
    let output = doob::output::format_human(&todos);

    // Verify output contains expected elements
    assert!(output.contains("Task 1"));
    assert!(output.contains("Task 2"));
    assert!(output.contains("priority: 1"));
    assert!(output.contains("priority: 2"));
    assert!(output.contains("Project: Project A"));
    assert!(output.contains("Tags: urgent, backend"));
    assert!(output.contains("[pending]"));
}

#[test]
fn test_human_format_completed_status() {
    use chrono::Utc;
    let todo = doob::models::Todo {
        id: None,
        uuid: "test-1".to_string(),
        content: "Done task".to_string(),
        status: doob::models::TodoStatus::Completed,
        priority: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        completed_at: None,
        due_date: None,
        project: None,
        project_path: None,
        file_path: None,
        tags: vec![],
        metadata: None,
        blocks: vec![],
        blocked_by: vec![],
    };
    let output = doob::output::format_human(&[todo]);
    assert!(output.contains("[completed]"));
    assert!(output.contains("Done task"));
}

#[test]
fn test_human_format_cancelled_status() {
    use chrono::Utc;
    let todo = doob::models::Todo {
        id: None,
        uuid: "test-2".to_string(),
        content: "Cancelled task".to_string(),
        status: doob::models::TodoStatus::Cancelled,
        priority: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        completed_at: None,
        due_date: None,
        project: None,
        project_path: None,
        file_path: None,
        tags: vec![],
        metadata: None,
        blocks: vec![],
        blocked_by: vec![],
    };
    let output = doob::output::format_human(&[todo]);
    assert!(output.contains("[cancelled]"));
}

#[tokio::test]
async fn test_list_empty() {
    let db = setup_test_db().await;

    let todos = doob::commands::list::execute(&db, None, None, None)
        .await
        .unwrap();
    let output = doob::output::format_human(&todos);

    assert_eq!(output, "No todos found");
}
