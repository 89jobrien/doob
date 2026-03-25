mod common;

use common::setup_test_db;

#[tokio::test]
async fn test_due_nonexistent_todo_errors() {
    let db = setup_test_db().await;

    let result = doob::commands::due::execute(
        &db,
        "nonexistent".to_string(),
        Some("2026-12-31".to_string()),
    )
    .await;

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("not found"));
}

#[tokio::test]
async fn test_due_rejects_invalid_date_format() {
    let db = setup_test_db().await;

    let created =
        doob::commands::add::execute(&db, vec!["Task".to_string()], None, None, None, None)
            .await
            .unwrap();

    let todo_id = created[0].id.as_ref().map(|t| t.to_string()).unwrap();

    let result = doob::commands::due::execute(&db, todo_id, Some("not-a-date".to_string())).await;

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Invalid date format"));
}

#[tokio::test]
async fn test_due_sets_valid_date() {
    let db = setup_test_db().await;

    let created = doob::commands::add::execute(
        &db,
        vec!["Task with due date".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let todo_id = created[0].id.as_ref().map(|t| t.to_string()).unwrap();

    let result =
        doob::commands::due::execute(&db, todo_id.clone(), Some("2026-12-31".to_string())).await;

    assert!(result.is_ok());

    let query = format!("SELECT * FROM {}", todo_id);
    let mut res = db.query(&query).await.unwrap();
    let todos: Vec<doob::models::Todo> = res.take(0).unwrap();
    assert!(todos[0].due_date.is_some());
}

#[tokio::test]
async fn test_due_clear_removes_due_date() {
    let db = setup_test_db().await;

    let created = doob::commands::add::execute(
        &db,
        vec!["Task to clear".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let todo_id = created[0].id.as_ref().map(|t| t.to_string()).unwrap();

    // Set a due date
    doob::commands::due::execute(&db, todo_id.clone(), Some("2026-12-31".to_string()))
        .await
        .unwrap();

    // Clear it
    let result =
        doob::commands::due::execute(&db, todo_id.clone(), Some("clear".to_string())).await;

    assert!(result.is_ok());

    let query = format!("SELECT * FROM {}", todo_id);
    let mut res = db.query(&query).await.unwrap();
    let todos: Vec<doob::models::Todo> = res.take(0).unwrap();
    assert!(todos[0].due_date.is_none());
}

#[tokio::test]
async fn test_due_none_clears_due_date() {
    let db = setup_test_db().await;

    let created = doob::commands::add::execute(
        &db,
        vec!["Task none due".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let todo_id = created[0].id.as_ref().map(|t| t.to_string()).unwrap();

    // Set a due date first
    doob::commands::due::execute(&db, todo_id.clone(), Some("2026-06-01".to_string()))
        .await
        .unwrap();

    // Pass None to clear
    let result = doob::commands::due::execute(&db, todo_id.clone(), None).await;

    assert!(result.is_ok());

    let query = format!("SELECT * FROM {}", todo_id);
    let mut res = db.query(&query).await.unwrap();
    let todos: Vec<doob::models::Todo> = res.take(0).unwrap();
    assert!(todos[0].due_date.is_none());
}
