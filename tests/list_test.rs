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
    )
    .await
    .unwrap();

    // Test list
    let todos = doob::commands::list::execute(&db, None, None, None).await;

    assert!(todos.is_ok());
    let todos = todos.unwrap();
    assert_eq!(todos.len(), 2);
}

#[tokio::test]
async fn test_list_filter_by_status() {
    let db = setup_test_db().await;

    // Add todos
    doob::commands::add::execute(&db, vec!["Task 1".to_string()], None, None, None, None)
        .await
        .unwrap();

    // Complete one
    let todos: Vec<doob::models::Todo> = db.select("todo").await.unwrap();
    let todo_id = todos[0].id.clone().unwrap().to_string();
    doob::commands::complete::execute(&db, vec![todo_id])
        .await
        .unwrap();

    // Add another pending
    doob::commands::add::execute(&db, vec!["Task 2".to_string()], None, None, None, None)
        .await
        .unwrap();

    // List only pending
    let pending = doob::commands::list::execute(&db, Some("pending".to_string()), None, None)
        .await
        .unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].content, "Task 2");
}

#[tokio::test]
async fn test_list_filter_by_project() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["Alpha task".to_string()],
        None,
        Some("project-alpha".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::add::execute(
        &db,
        vec!["Beta task".to_string()],
        None,
        Some("project-beta".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    let result = doob::commands::list::execute(&db, None, Some("project-alpha".to_string()), None)
        .await
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].content, "Alpha task");
}

#[tokio::test]
async fn test_list_with_limit() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec![
            "T1".to_string(),
            "T2".to_string(),
            "T3".to_string(),
            "T4".to_string(),
            "T5".to_string(),
        ],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let result = doob::commands::list::execute(&db, None, None, Some(2))
        .await
        .unwrap();

    assert_eq!(result.len(), 2);
}
