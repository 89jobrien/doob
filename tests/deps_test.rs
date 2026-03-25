mod common;
use common::setup_test_db;

#[tokio::test]
async fn test_deps_no_dependencies() {
    let db = setup_test_db().await;

    let todos = doob::commands::add::execute(
        &db,
        vec!["Standalone task".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let view = doob::commands::deps::execute(&db, todos[0].uuid.clone())
        .await
        .unwrap();

    assert_eq!(view.root.uuid, todos[0].uuid);
    assert!(view.blockers.is_empty());
    assert!(view.dependents.is_empty());
}

#[tokio::test]
async fn test_deps_link_blocked_by() {
    let db = setup_test_db().await;

    let blocker = doob::commands::add::execute(
        &db,
        vec!["Build the thing".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let blocked = doob::commands::add::execute(
        &db,
        vec!["Deploy the thing".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::deps::link(&db, &blocked[0].uuid, &[], &[blocker[0].uuid.clone()])
        .await
        .unwrap();

    let view = doob::commands::deps::execute(&db, blocked[0].uuid.clone())
        .await
        .unwrap();

    assert_eq!(view.blockers.len(), 1);
    assert_eq!(view.blockers[0].uuid, blocker[0].uuid);
    assert!(view.dependents.is_empty());
}

#[tokio::test]
async fn test_deps_link_blocks() {
    let db = setup_test_db().await;

    let blocker =
        doob::commands::add::execute(&db, vec!["Build first".to_string()], None, None, None, None)
            .await
            .unwrap();

    let dependent = doob::commands::add::execute(
        &db,
        vec!["Deploy after".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::deps::link(&db, &blocker[0].uuid, &[dependent[0].uuid.clone()], &[])
        .await
        .unwrap();

    let view = doob::commands::deps::execute(&db, blocker[0].uuid.clone())
        .await
        .unwrap();

    assert!(view.blockers.is_empty());
    assert_eq!(view.dependents.len(), 1);
    assert_eq!(view.dependents[0].uuid, dependent[0].uuid);
}

#[tokio::test]
async fn test_deps_not_found_returns_error() {
    let db = setup_test_db().await;

    let result =
        doob::commands::deps::execute(&db, "00000000-0000-0000-0000-000000000000".to_string())
            .await;

    assert!(result.is_err());
}
