mod common;

use common::setup_test_db;
use doob::models::TodoStatus;
use doob::output::kanban::render_board;

#[tokio::test]
async fn test_kan_empty() {
    let db = setup_test_db().await;

    let (todos, filter) = doob::commands::kan::execute(&db, None, None).await.unwrap();

    let board = render_board(&todos, filter.as_deref());
    assert!(board.contains("No todos found"));
}

#[tokio::test]
async fn test_kan_groups_by_project() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["Write tests".to_string()],
        None,
        Some("doob".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::add::execute(
        &db,
        vec!["Fix bug".to_string()],
        None,
        Some("doob".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::add::execute(
        &db,
        vec!["Other task".to_string()],
        None,
        Some("other".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    let (todos, filter) = doob::commands::kan::execute(&db, None, None).await.unwrap();

    assert_eq!(todos.len(), 3);
    let board = render_board(&todos, filter.as_deref());
    assert!(board.contains("project: doob"));
    assert!(board.contains("project: other"));
}

#[tokio::test]
async fn test_kan_project_filter() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["Alpha task".to_string()],
        None,
        Some("alpha".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::add::execute(
        &db,
        vec!["Beta task".to_string()],
        None,
        Some("beta".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    let (todos, filter) = doob::commands::kan::execute(&db, Some("alpha".to_string()), None)
        .await
        .unwrap();

    assert_eq!(todos.len(), 1);
    let board = render_board(&todos, filter.as_deref());
    assert!(board.contains("Alpha task"));
    assert!(!board.contains("Beta task"));
}

#[tokio::test]
async fn test_kan_status_filter() {
    let db = setup_test_db().await;

    let created = doob::commands::add::execute(
        &db,
        vec!["Task to complete".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let id = created[0].id.clone().unwrap();

    doob::commands::complete::execute(&db, vec![id])
        .await
        .unwrap();

    doob::commands::add::execute(
        &db,
        vec!["Pending task".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let (todos, _) = doob::commands::kan::execute(&db, None, None).await.unwrap();

    let filter = vec![TodoStatus::Pending];
    let board = render_board(&todos, Some(&filter));
    assert!(board.contains("Pending task"));
    assert!(!board.contains("Task to complete"));
}

#[tokio::test]
async fn test_render_board_truncates_long_content() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["This is a very long task description that should be truncated".to_string()],
        None,
        Some("test".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    let (todos, filter) = doob::commands::kan::execute(&db, None, None).await.unwrap();

    let board = render_board(&todos, filter.as_deref());
    assert!(board.contains("..."));
}

#[test]
fn test_render_board_no_matching_todos_after_filter() {
    use chrono::Utc;
    use doob::models::{Todo, TodoStatus};
    use doob::output::kanban::render_board;

    let todo = Todo {
        id: None,
        uuid: "test-uuid-1".to_string(),
        content: "Pending task".to_string(),
        status: TodoStatus::Pending,
        priority: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        completed_at: None,
        due_date: None,
        project: Some("my-project".to_string()),
        project_path: None,
        file_path: None,
        tags: vec![],
        metadata: None,
        blocks: vec![],
        blocked_by: vec![],
    };

    // Filter for Completed only — no todos match
    let filter = vec![TodoStatus::Completed];
    let board = render_board(&[todo], Some(&filter));
    assert!(board.contains("(no matching todos)"));
}
