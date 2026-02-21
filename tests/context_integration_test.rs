mod common;

use std::env;
use tempfile::TempDir;
use git2::Repository;
use doob::db;
use doob::commands::add;

#[tokio::test]
#[cfg_attr(not(feature = "no_parallel"), serial_test::serial)]
async fn test_context_detection_integration() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    let repo = Repository::init(repo_path).unwrap();

    // Add remote
    repo.remote("origin", "git@github.com:user/my-project.git").unwrap();

    // Change to repo directory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(repo_path).unwrap();

    // Create database
    let db = db::create_connection(None).await.unwrap();

    // Test 1: Add todo without explicit project (should auto-detect)
    let todos = add::execute(
        &db,
        vec!["Test auto-detect".to_string()],
        None,
        None, // No project provided
        None, // No file_path provided
        None,
    ).await.unwrap();

    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].project, Some("my-project".to_string()));
    assert_eq!(todos[0].file_path, None); // At root, so no file_path

    // Test 2: User-provided project should override detection
    let todos = add::execute(
        &db,
        vec!["Test override".to_string()],
        None,
        Some("explicit-project".to_string()),
        None,
        None,
    ).await.unwrap();

    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].project, Some("explicit-project".to_string()));

    // Restore directory before test ends
    env::set_current_dir(&original_dir).unwrap();
}

#[tokio::test]
#[cfg_attr(not(feature = "no_parallel"), serial_test::serial)]
async fn test_file_path_detection() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    let repo = Repository::init(repo_path).unwrap();
    repo.remote("origin", "git@github.com:user/test-repo.git").unwrap();

    // Create subdirectory
    let sub_dir = repo_path.join("src").join("components");
    std::fs::create_dir_all(&sub_dir).unwrap();

    // Change to subdirectory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&sub_dir).unwrap();

    // Create database
    let db = db::create_connection(None).await.unwrap();

    // Add todo from subdirectory
    let todos = add::execute(
        &db,
        vec!["Test from subdir".to_string()],
        None,
        None,
        None,
        None,
    ).await.unwrap();

    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].project, Some("test-repo".to_string()));
    assert_eq!(todos[0].file_path, Some("src/components".to_string()));

    // Restore directory before test ends
    env::set_current_dir(&original_dir).unwrap();
}
