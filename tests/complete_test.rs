mod common;

use common::setup_test_db;
use doob::models::TodoStatus;

#[tokio::test]
async fn test_complete_single_todo() {
    let db = setup_test_db().await;

    // Create a todo
    doob::commands::add::execute(&db, vec!["Test".to_string()], None, None, None, None).await.unwrap();

    // Get the ID
    let todos: Vec<doob::models::Todo> = db.select("todo").await.unwrap();
    let todo_id = todos[0].id.clone().unwrap().to_string();

    // Complete it
    let result = doob::commands::complete::execute(&db, vec![todo_id.clone()]).await;
    if let Err(e) = &result {
        eprintln!("Error completing todo: {}", e);
    }
    assert!(result.is_ok());

    // Verify status changed - use query instead of select
    let query = format!("SELECT * FROM {}", todo_id);
    let mut result = db.query(&query).await.unwrap();
    let updated: Vec<doob::models::Todo> = result.take(0).unwrap();
    assert!(!updated.is_empty());
    let updated = &updated[0];
    assert_eq!(updated.status, TodoStatus::Completed);
    assert!(updated.completed_at.is_some());
}

#[tokio::test]
async fn test_complete_batch_todos() {
    let db = setup_test_db().await;

    // Create 3 todos
    doob::commands::add::execute(
        &db,
        vec!["T1".to_string(), "T2".to_string(), "T3".to_string()],
        None,
        None,
        None,
        None,
    ).await.unwrap();

    // Get IDs
    let todos: Vec<doob::models::Todo> = db.select("todo").await.unwrap();
    let ids: Vec<String> = todos.iter().map(|t| t.id.clone().unwrap().to_string()).collect();

    // Complete all
    let result = doob::commands::complete::execute(&db, ids).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);

    // Verify all completed
    let all: Vec<doob::models::Todo> = db.select("todo").await.unwrap();
    assert!(all.iter().all(|t| t.status == TodoStatus::Completed));
}
