mod common;

use common::setup_test_db;

#[tokio::test]
async fn test_list_all_todos() {
    let db = setup_test_db().await;

    // Add some test data
    doob::commands::add::execute(
        &db,
        vec!["Task 1".to_string(), "Task 2".to_string()],
        Some(1),
        None,
        None,
        None,
    ).await.unwrap();

    // Test list
    let todos = doob::commands::list::execute(&db, None, None, None).await;

    assert!(todos.is_ok());
    let todos = todos.unwrap();
    assert_eq!(todos.len(), 2);
}

// Note: This test is commented out as it requires the complete command (Task 6)
// Uncomment after implementing the complete command

/*
use doob::models::TodoStatus;

#[tokio::test]
async fn test_list_filter_by_status() {
    let db = setup_test_db().await;

    // Add todos
    doob::commands::add::execute(&db, vec!["Task 1".to_string()], None, None, None, None).await.unwrap();

    // Complete one
    let todos: Vec<doob::models::Todo> = db.select("todo").await.unwrap();
    let todo_id = todos[0].id.clone().unwrap().to_string();
    doob::commands::complete::execute(&db, vec![todo_id]).await.unwrap();

    // Add another pending
    doob::commands::add::execute(&db, vec!["Task 2".to_string()], None, None, None, None).await.unwrap();

    // List only pending
    let pending = doob::commands::list::execute(&db, Some("pending".to_string()), None, None).await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].content, "Task 2");
}
*/
