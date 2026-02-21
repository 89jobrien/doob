use std::env;
use git2::Repository;

pub fn detect_project() -> Option<String> {
    // Try to find git repository
    let cwd = env::current_dir().ok()?;
    let repo = Repository::discover(&cwd).ok()?;

    // Get remote URL
    let remote = repo.find_remote("origin").ok()?;
    let url = remote.url()?;

    // Parse project name from URL
    // Examples:
    //   git@github.com:user/project.git -> project
    //   https://github.com/user/project -> project
    let name = url
        .split('/')
        .last()?
        .trim_end_matches(".git");

    Some(name.to_string())
}

pub fn detect_file_path() -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let repo = Repository::discover(&cwd).ok()?;
    let workdir = repo.workdir()?;

    // Get relative path from repo root to cwd
    let rel_path = cwd.strip_prefix(workdir).ok()?;

    if rel_path.as_os_str().is_empty() {
        None
    } else {
        Some(rel_path.to_string_lossy().to_string())
    }
}
