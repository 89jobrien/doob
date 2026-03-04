mod common;

use common::setup_test_db;

#[tokio::test]
async fn test_add_single_note() {
    let db = setup_test_db().await;

    let result = doob::commands::note::add::execute(
        &db,
        vec!["Remember to update changelog".to_string()],
        None,
        None,
        None,
    )
    .await;

    assert!(result.is_ok());
    let notes = result.unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Remember to update changelog");
    assert!(notes[0].tags.is_empty());
}

#[tokio::test]
async fn test_add_note_with_project_and_tags() {
    let db = setup_test_db().await;

    let result = doob::commands::note::add::execute(
        &db,
        vec!["Meeting notes".to_string()],
        Some("doob".to_string()),
        None,
        Some("work,meeting".to_string()),
    )
    .await;

    assert!(result.is_ok());
    let notes = result.unwrap();
    assert_eq!(notes[0].project, Some("doob".to_string()));
    assert_eq!(notes[0].tags, vec!["work", "meeting"]);
}

#[tokio::test]
async fn test_list_notes() {
    let db = setup_test_db().await;

    doob::commands::note::add::execute(
        &db,
        vec!["Note A".to_string(), "Note B".to_string()],
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let notes = doob::commands::note::list::execute(&db, None, None)
        .await
        .unwrap();

    assert_eq!(notes.len(), 2);
}

#[tokio::test]
async fn test_list_notes_by_project() {
    let db = setup_test_db().await;

    doob::commands::note::add::execute(
        &db,
        vec!["Project note".to_string()],
        Some("alpha".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    doob::commands::note::add::execute(
        &db,
        vec!["Other note".to_string()],
        Some("beta".to_string()),
        None,
        None,
    )
    .await
    .unwrap();

    let notes = doob::commands::note::list::execute(&db, Some("alpha".to_string()), None)
        .await
        .unwrap();

    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Project note");
}

#[tokio::test]
async fn test_list_notes_with_limit() {
    let db = setup_test_db().await;

    doob::commands::note::add::execute(
        &db,
        vec!["N1".to_string(), "N2".to_string(), "N3".to_string()],
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let notes = doob::commands::note::list::execute(&db, None, Some(2))
        .await
        .unwrap();

    assert_eq!(notes.len(), 2);
}

#[tokio::test]
async fn test_remove_note() {
    let db = setup_test_db().await;

    let created = doob::commands::note::add::execute(
        &db,
        vec!["Temporary note".to_string()],
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let id = created[0]
        .id
        .as_ref()
        .map(|t| t.id.to_string())
        .unwrap();

    let count = doob::commands::note::remove::execute(&db, vec![id])
        .await
        .unwrap();

    assert_eq!(count, 1);

    let remaining = doob::commands::note::list::execute(&db, None, None)
        .await
        .unwrap();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn test_remove_nonexistent_note_errors() {
    let db = setup_test_db().await;

    let result = doob::commands::note::remove::execute(&db, vec!["nonexistent_id".to_string()])
        .await;

    assert!(result.is_err());
}
