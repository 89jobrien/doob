mod common;

use std::env;
use tempfile::TempDir;
use git2::Repository;

#[tokio::test]
async fn test_detect_project_from_git() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    let repo = Repository::init(repo_path).unwrap();

    // Add remote
    repo.remote("origin", "git@github.com:user/test-project.git").unwrap();

    // Change to repo directory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(repo_path).unwrap();

    // Test detection
    let project = doob::context::detect_project();
    assert!(project.is_some());
    assert_eq!(project.unwrap(), "test-project");

    // Restore directory
    env::set_current_dir(original_dir).unwrap();
}
