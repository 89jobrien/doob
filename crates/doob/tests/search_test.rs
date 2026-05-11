mod common;
use common::setup_test_db;

#[tokio::test]
async fn test_search_finds_todo_by_content() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["Refactor the database layer".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let results =
        doob::commands::search::execute(&db, "refactor".to_string(), "all".to_string(), None)
            .await
            .unwrap();

    assert_eq!(results.todos.len(), 1);
    assert_eq!(results.notes.len(), 0);
    assert!(results.todos[0].content.contains("Refactor"));
}

#[tokio::test]
async fn test_search_finds_note_by_content() {
    let db = setup_test_db().await;

    doob::commands::note::add::execute(
        &db,
        vec!["Remember to refactor auth module".to_string()],
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let results =
        doob::commands::search::execute(&db, "refactor".to_string(), "all".to_string(), None)
            .await
            .unwrap();

    assert_eq!(results.todos.len(), 0);
    assert_eq!(results.notes.len(), 1);
}

#[tokio::test]
async fn test_search_finds_both_types() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["Deploy to production".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::note::add::execute(
        &db,
        vec!["Deploy checklist item".to_string()],
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let results =
        doob::commands::search::execute(&db, "deploy".to_string(), "all".to_string(), None)
            .await
            .unwrap();

    assert_eq!(results.todos.len(), 1);
    assert_eq!(results.notes.len(), 1);
}

#[tokio::test]
async fn test_search_type_filter_todo_only() {
    let db = setup_test_db().await;

    doob::commands::add::execute(&db, vec!["Fix the bug".to_string()], None, None, None, None)
        .await
        .unwrap();
    doob::commands::note::add::execute(
        &db,
        vec!["Note about the bug".to_string()],
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let results = doob::commands::search::execute(&db, "bug".to_string(), "todo".to_string(), None)
        .await
        .unwrap();

    assert_eq!(results.todos.len(), 1);
    assert_eq!(results.notes.len(), 0);
}

#[tokio::test]
async fn test_search_no_results_returns_empty() {
    let db = setup_test_db().await;

    let results =
        doob::commands::search::execute(&db, "zzznomatch".to_string(), "all".to_string(), None)
            .await
            .unwrap();

    assert!(results.todos.is_empty());
    assert!(results.notes.is_empty());
}

#[tokio::test]
async fn test_search_project_filter() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["Deploy alpha".to_string()],
        None,
        Some("alpha".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::add::execute(
        &db,
        vec!["Deploy beta".to_string()],
        None,
        Some("beta".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    let results = doob::commands::search::execute(
        &db,
        "deploy".to_string(),
        "todo".to_string(),
        Some("alpha".to_string()),
    )
    .await
    .unwrap();

    assert_eq!(results.todos.len(), 1);
    assert_eq!(results.todos[0].project, Some("alpha".to_string()));
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["UPPERCASE TASK".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let results =
        doob::commands::search::execute(&db, "uppercase".to_string(), "todo".to_string(), None)
            .await
            .unwrap();

    assert_eq!(results.todos.len(), 1);
}
