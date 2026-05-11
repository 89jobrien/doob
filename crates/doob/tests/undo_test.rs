mod common;

use common::setup_test_db;

#[tokio::test]
async fn test_undo_nonexistent_todo_errors() {
    let db = setup_test_db().await;

    let result = doob::commands::undo::execute(&db, vec!["nonexistent".to_string()]).await;

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("not found"));
}

#[tokio::test]
async fn test_undo_pending_todo_errors() {
    let db = setup_test_db().await;

    let created = doob::commands::add::execute(
        &db,
        vec!["A pending task".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let todo_id = created[0].id.as_ref().map(|t| t.to_string()).unwrap();

    // Try to undo a todo that's pending (not completed)
    let result = doob::commands::undo::execute(&db, vec![todo_id]).await;

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("not completed"));
}

#[tokio::test]
async fn test_undo_completed_todo_resets_to_pending() {
    let db = setup_test_db().await;

    // Create and complete a todo
    let created = doob::commands::add::execute(
        &db,
        vec!["Task to undo".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let todo_id = created[0].id.as_ref().map(|t| t.to_string()).unwrap();

    doob::commands::complete::execute(&db, vec![todo_id.clone()])
        .await
        .unwrap();

    // Verify it's completed
    let query = format!("SELECT * FROM {}", todo_id);
    let mut res = db.query(&query).await.unwrap();
    let todos: Vec<doob::models::Todo> = res.take(0).unwrap();
    assert_eq!(todos[0].status, doob::models::TodoStatus::Completed);

    // Undo it
    let result = doob::commands::undo::execute(&db, vec![todo_id.clone()]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);

    // Verify it's pending again
    let mut res2 = db.query(&query).await.unwrap();
    let todos2: Vec<doob::models::Todo> = res2.take(0).unwrap();
    assert_eq!(todos2[0].status, doob::models::TodoStatus::Pending);
}
