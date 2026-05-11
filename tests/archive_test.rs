mod common;
use common::setup_test_db;

#[tokio::test]
async fn test_archive_dry_run_returns_candidates_without_moving() {
    let db = setup_test_db().await;

    let todos = doob::commands::add::execute(
        &db,
        vec!["Old completed task".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let todo_id = todos[0].id.clone().unwrap();
    doob::commands::complete::execute(&db, vec![todo_id])
        .await
        .unwrap();

    // Force updated_at to be in the past via raw query
    db.query(format!(
        "UPDATE todo SET updated_at = <datetime>'2020-01-01T00:00:00.000Z' WHERE uuid = '{}'",
        todos[0].uuid
    ))
    .await
    .unwrap();

    // Dry run — should find candidate
    let result = doob::commands::archive::run::execute(&db, 1, false, None)
        .await
        .unwrap();

    assert!(result.dry_run);
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.archived_count, 0);

    // Todo should still be in the active table
    let remaining = doob::commands::list::execute(&db, None, None, None)
        .await
        .unwrap();
    assert_eq!(remaining.len(), 1);
}

#[tokio::test]
async fn test_archive_apply_moves_todo() {
    let db = setup_test_db().await;

    let todos = doob::commands::add::execute(
        &db,
        vec!["Old task to archive".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let todo_id = todos[0].id.clone().unwrap();
    doob::commands::complete::execute(&db, vec![todo_id])
        .await
        .unwrap();

    db.query(format!(
        "UPDATE todo SET updated_at = <datetime>'2020-01-01T00:00:00.000Z' WHERE uuid = '{}'",
        todos[0].uuid
    ))
    .await
    .unwrap();

    let result = doob::commands::archive::run::execute(&db, 1, true, None)
        .await
        .unwrap();

    assert!(!result.dry_run);
    assert_eq!(result.archived_count, 1);

    // Should be gone from active table
    let active = doob::commands::list::execute(&db, None, None, None)
        .await
        .unwrap();
    assert!(active.is_empty());

    // Should appear in archive list
    let archived = doob::commands::archive::list::execute(&db, None, None)
        .await
        .unwrap();
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].content, "Old task to archive");
}

#[tokio::test]
async fn test_archive_skips_pending_todos() {
    let db = setup_test_db().await;

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

    let result = doob::commands::archive::run::execute(&db, 0, false, None)
        .await
        .unwrap();

    assert!(result.candidates.is_empty());
}

#[tokio::test]
async fn test_archive_list_project_filter() {
    let db = setup_test_db().await;

    let a = doob::commands::add::execute(
        &db,
        vec!["Alpha task".to_string()],
        None,
        Some("alpha".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    let b = doob::commands::add::execute(
        &db,
        vec!["Beta task".to_string()],
        None,
        Some("beta".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    for todo in [&a[0], &b[0]] {
        let record_id = todo.id.clone().unwrap();
        doob::commands::complete::execute(&db, vec![record_id])
            .await
            .unwrap();
        db.query(format!(
            "UPDATE todo SET updated_at = <datetime>'2020-01-01T00:00:00.000Z' WHERE uuid = '{}'",
            todo.uuid
        ))
        .await
        .unwrap();
    }

    doob::commands::archive::run::execute(&db, 1, true, None)
        .await
        .unwrap();

    let archived = doob::commands::archive::list::execute(&db, Some("alpha".to_string()), None)
        .await
        .unwrap();

    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].project, Some("alpha".to_string()));
}
