mod common;
use common::setup_test_db;

/// Regression test for GH #7: batch add with --blocks/--blocked-by must only
/// link deps to the first created todo, not to every todo in the batch.
#[tokio::test]
async fn test_batch_add_deps_only_linked_to_first_todo() {
    let db = setup_test_db().await;

    // Create the blocker todo
    let blocker = doob::commands::add::execute(
        &db,
        vec!["Gate task".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Simulate a batch add of two todos with --blocked-by pointing at blocker
    let batch = doob::commands::add::execute(
        &db,
        vec!["First task".to_string(), "Second task".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // apply_batch_deps should only link the first todo in the batch
    doob::commands::deps::apply_batch_deps(
        &db,
        &batch,
        &[],
        &[blocker[0].uuid.clone()],
    )
    .await
    .unwrap();

    // First todo must be blocked by the gate task
    let view_first = doob::commands::deps::execute(&db, batch[0].uuid.clone())
        .await
        .unwrap();
    assert_eq!(view_first.blockers.len(), 1, "first todo must have one blocker");

    // Second todo must NOT be linked to any deps
    let view_second = doob::commands::deps::execute(&db, batch[1].uuid.clone())
        .await
        .unwrap();
    assert!(
        view_second.blockers.is_empty(),
        "second todo must NOT have blockers — only the first todo should be linked"
    );
}

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
