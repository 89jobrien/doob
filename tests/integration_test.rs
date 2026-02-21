mod common;

use common::setup_test_db;
use doob::models::TodoStatus;

#[tokio::test]
async fn test_complete_workflow() {
    let db = setup_test_db().await;

    // Add a todo
    doob::commands::add::execute(&db, vec!["Test task".to_string()], None, None, None, None)
        .await
        .unwrap();

    // List todos
    let todos = doob::commands::list::execute(&db, None, None, None)
        .await
        .unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].status, TodoStatus::Pending);

    // Complete it
    let id = todos[0].id.clone().unwrap().to_string();
    let count = doob::commands::complete::execute(&db, vec![id]).await.unwrap();
    assert_eq!(count, 1);

    // List again
    let todos = doob::commands::list::execute(&db, None, None, None)
        .await
        .unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].status, TodoStatus::Completed);
    assert!(todos[0].completed_at.is_some());
}
