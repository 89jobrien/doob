use doob_core::models::handoff::HandoffState;
use doob_core::ports::{HandoffRepository, HandoffSessionRepository, TodoRepository};
use doob_sqlite::{
    create_connection, HandoffRepositoryImpl, HandoffSessionRepositoryImpl, TodoRepositoryImpl,
};
use tempfile::tempdir;

fn test_conn() -> doob_sqlite::SqliteConnection {
    let tmp = tempdir().expect("tempdir");
    let db_path = tmp.keep().join("test.db");
    create_connection(Some(db_path.to_str().unwrap())).expect("create_connection")
}

#[tokio::test]
async fn todo_crud_round_trip() {
    let conn = test_conn();
    let repo = TodoRepositoryImpl::new(conn);

    // Create
    let todos = repo
        .create_todos(vec![(
            "Test todo".into(),
            uuid::Uuid::new_v4().to_string(),
            3,
            Some("test-project".into()),
            None,
            vec!["tag1".into()],
        )])
        .await
        .expect("create");
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].content, "Test todo");
    assert_eq!(todos[0].priority, 3);

    let record_id = todos[0].id.as_ref().unwrap().clone();

    // Read
    let fetched = repo.get_todo(&record_id).await.expect("get");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().content, "Test todo");

    // List
    let all = repo.list_todos(None, None, None).await.expect("list");
    assert_eq!(all.len(), 1);

    // Complete
    repo.complete_todo(&record_id).await.expect("complete");
    let completed = repo.get_todo(&record_id).await.expect("get").unwrap();
    assert_eq!(
        completed.status,
        doob_core::models::todo::TodoStatus::Completed
    );

    // Undo
    repo.undo_todo(&record_id).await.expect("undo");
    let undone = repo.get_todo(&record_id).await.expect("get").unwrap();
    assert_eq!(undone.status, doob_core::models::todo::TodoStatus::Pending);

    // Delete
    repo.delete_todo(&record_id).await.expect("delete");
    let gone = repo.get_todo(&record_id).await.expect("get");
    assert!(gone.is_none());
}

#[tokio::test]
async fn handoff_status_update() {
    let conn = test_conn();
    let repo = HandoffRepositoryImpl::new(conn.clone());

    // Insert a handoff item directly via SQL
    conn.with_conn(|c| {
        c.execute(
            "INSERT INTO handoff_item (uuid, handoff_id, project, title, priority, status)
             VALUES ('test-uuid', 'hj-1', 'test', 'Test item', 'P1', 'open')",
            [],
        )?;
        Ok(())
    })
    .expect("insert");

    // Verify it exists
    let item = repo
        .get_by_handoff_id("hj-1")
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(item.status, "open");

    // Update status
    repo.update_handoff_status("hj-1", "blocked")
        .await
        .expect("update");
    let updated = repo
        .get_by_handoff_id("hj-1")
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(updated.status, "blocked");

    // Complete
    repo.update_handoff_status("hj-1", "done")
        .await
        .expect("done");
    let done = repo
        .get_by_handoff_id("hj-1")
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(done.status, "done");
    assert!(done.completed_at.is_some());
}

#[tokio::test]
async fn handoff_list_filters() {
    let conn = test_conn();
    let repo = HandoffRepositoryImpl::new(conn.clone());

    conn.with_conn(|c| {
        c.execute_batch(
            "INSERT INTO handoff_item (uuid, handoff_id, project, title, priority, status)
             VALUES ('u1', 'hj-1', 'alpha', 'Item 1', 'P1', 'open');
             INSERT INTO handoff_item (uuid, handoff_id, project, title, priority, status)
             VALUES ('u2', 'hj-2', 'alpha', 'Item 2', 'P2', 'blocked');
             INSERT INTO handoff_item (uuid, handoff_id, project, title, priority, status)
             VALUES ('u3', 'hj-3', 'beta', 'Item 3', 'P1', 'open');",
        )?;
        Ok(())
    })
    .expect("insert");

    let all = repo.list_handoff_items(None, None).await.expect("list all");
    assert_eq!(all.len(), 3);

    let alpha = repo
        .list_handoff_items(Some("alpha"), None)
        .await
        .expect("list alpha");
    assert_eq!(alpha.len(), 2);

    let open = repo
        .list_handoff_items(None, Some("open"))
        .await
        .expect("list open");
    assert_eq!(open.len(), 2);
}

#[tokio::test]
async fn todo_search() {
    let conn = test_conn();
    let repo = TodoRepositoryImpl::new(conn);

    repo.create_todos(vec![
        (
            "Fix the build".into(),
            uuid::Uuid::new_v4().to_string(),
            4,
            Some("proj".into()),
            None,
            vec![],
        ),
        (
            "Write docs".into(),
            uuid::Uuid::new_v4().to_string(),
            2,
            Some("proj".into()),
            None,
            vec![],
        ),
    ])
    .await
    .expect("create");

    let results = repo.search_todos("build", None).await.expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "Fix the build");
}

#[tokio::test]
async fn session_log_round_trip() {
    let conn = test_conn();
    let repo = HandoffSessionRepositoryImpl::new(conn);

    repo.log_append("proj", "2026-05-11", "did stuff", &["abc123".into()])
        .await
        .expect("append");
    repo.log_append("proj", "2026-05-12", "more stuff", &[])
        .await
        .expect("append");

    let entries = repo.log_query("proj").await.expect("query");
    assert_eq!(entries.len(), 2);
    // Most recent first
    assert_eq!(entries[0].date.as_deref(), Some("2026-05-12"));
    assert_eq!(entries[1].summary, "did stuff");
    assert_eq!(entries[1].commits.len(), 1);

    // Different project is isolated
    let empty = repo.log_query("other").await.expect("query");
    assert!(empty.is_empty());
}

#[tokio::test]
async fn session_state_save_load() {
    let conn = test_conn();
    let repo = HandoffSessionRepositoryImpl::new(conn);

    let state = HandoffState {
        branch: Some("main".into()),
        build: Some("clean".into()),
        tests: Some("passing".into()),
        notes: Some("all good".into()),
        ..HandoffState::default()
    };

    repo.save_state("proj", &state).await.expect("save");
    let loaded = repo
        .load_state("proj")
        .await
        .expect("load")
        .expect("exists");
    assert_eq!(loaded.branch.as_deref(), Some("main"));
    assert_eq!(loaded.build.as_deref(), Some("clean"));
    assert_eq!(loaded.notes.as_deref(), Some("all good"));

    // Upsert overwrites
    let state2 = HandoffState {
        branch: Some("feat/x".into()),
        build: Some("failing".into()),
        ..HandoffState::default()
    };
    repo.save_state("proj", &state2).await.expect("save");
    let loaded2 = repo
        .load_state("proj")
        .await
        .expect("load")
        .expect("exists");
    assert_eq!(loaded2.branch.as_deref(), Some("feat/x"));
    assert_eq!(loaded2.build.as_deref(), Some("failing"));

    // Missing project returns None
    let missing = repo.load_state("nope").await.expect("load");
    assert!(missing.is_none());
}
