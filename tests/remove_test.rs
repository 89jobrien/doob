mod common;

use common::setup_test_db;

#[tokio::test]
async fn test_remove_nonexistent_todo_errors() {
    let db = setup_test_db().await;

    let result = doob::commands::remove::execute(&db, vec!["nonexistent".to_string()]).await;

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("not found"));
}

#[tokio::test]
async fn test_remove_single_todo() {
    let db = setup_test_db().await;

    let created = doob::commands::add::execute(
        &db,
        vec!["Task to remove".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let todo_id = created[0].id.as_ref().map(|t| t.id.to_string()).unwrap();

    let result = doob::commands::remove::execute(&db, vec![todo_id]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);

    // Verify it's gone
    let remaining: Vec<doob::models::Todo> = db.select("todo").await.unwrap();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn test_remove_batch_todos() {
    let db = setup_test_db().await;

    let created = doob::commands::add::execute(
        &db,
        vec!["T1".to_string(), "T2".to_string(), "T3".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let ids: Vec<String> = created
        .iter()
        .map(|t| t.id.as_ref().map(|th| th.id.to_string()).unwrap())
        .collect();

    let result = doob::commands::remove::execute(&db, ids).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);

    let remaining: Vec<doob::models::Todo> = db.select("todo").await.unwrap();
    assert!(remaining.is_empty());
}
