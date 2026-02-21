mod common;

use common::setup_test_db;
use doob::models::TodoStatus;

#[tokio::test]
async fn test_add_single_todo() {
    let db = setup_test_db().await;

    let result = doob::commands::add::execute(
        &db,
        vec!["Fix auth bug".to_string()],
        Some(1),
        None,
        None,
        None,
    ).await;

    assert!(result.is_ok());

    // Query database to verify
    let todos: Vec<doob::models::Todo> = db
        .select("todo")
        .await
        .expect("Failed to query todos");

    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].content, "Fix auth bug");
    assert_eq!(todos[0].status, TodoStatus::Pending);
    assert_eq!(todos[0].priority, 1);
}

#[tokio::test]
async fn test_add_batch_todos() {
    let db = setup_test_db().await;

    let result = doob::commands::add::execute(
        &db,
        vec!["Task 1".to_string(), "Task 2".to_string(), "Task 3".to_string()],
        Some(2),
        None,
        None,
        Some("urgent,test".to_string()),
    ).await;

    assert!(result.is_ok());
    let created = result.unwrap();
    assert_eq!(created.len(), 3);

    // Verify in database
    let todos: Vec<doob::models::Todo> = db.select("todo").await.unwrap();
    assert_eq!(todos.len(), 3);
    assert_eq!(todos[0].tags, vec!["urgent", "test"]);
}
