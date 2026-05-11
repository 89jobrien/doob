mod common;
use common::setup_test_db;

#[tokio::test]
async fn test_stats_empty_db() {
    let db = setup_test_db().await;

    let stats = doob::commands::stats::execute(&db, None, 7).await.unwrap();

    assert_eq!(stats.total, 0);
    assert_eq!(stats.pending, 0);
    assert_eq!(stats.completed, 0);
    assert_eq!(stats.completion_rate, 0.0);
    assert!(stats.avg_completion_secs.is_none());
    assert_eq!(stats.overdue_count, 0);
}

#[tokio::test]
async fn test_stats_counts_by_status() {
    let db = setup_test_db().await;

    doob::commands::add::execute(
        &db,
        vec!["Task 1".to_string(), "Task 2".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let created =
        doob::commands::add::execute(&db, vec!["Task 3".to_string()], None, None, None, None)
            .await
            .unwrap();

    let record_id = created[0].id.clone().unwrap();
    doob::commands::complete::execute(&db, vec![record_id])
        .await
        .unwrap();

    let stats = doob::commands::stats::execute(&db, None, 7).await.unwrap();

    assert_eq!(stats.total, 3);
    assert_eq!(stats.pending, 2);
    assert_eq!(stats.completed, 1);
}

#[tokio::test]
async fn test_stats_completion_rate() {
    let db = setup_test_db().await;

    let todos = doob::commands::add::execute(
        &db,
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let record_id = todos[0].id.clone().unwrap();
    doob::commands::complete::execute(&db, vec![record_id])
        .await
        .unwrap();

    let stats = doob::commands::stats::execute(&db, None, 7).await.unwrap();

    assert_eq!(stats.total, 3);
    assert_eq!(stats.completed, 1);
    assert!((stats.completion_rate - 33.333).abs() < 0.1);
}

#[tokio::test]
async fn test_stats_window_counts_recent() {
    let db = setup_test_db().await;

    doob::commands::add::execute(&db, vec!["Recent task".to_string()], None, None, None, None)
        .await
        .unwrap();

    let stats = doob::commands::stats::execute(&db, None, 7).await.unwrap();

    assert_eq!(stats.created_window, 1);
    assert_eq!(stats.window_days, 7);
}

#[tokio::test]
async fn test_stats_project_filter() {
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

    let stats = doob::commands::stats::execute(&db, Some("alpha".to_string()), 7)
        .await
        .unwrap();

    assert_eq!(stats.total, 1);
    assert_eq!(stats.project, Some("alpha".to_string()));
}
