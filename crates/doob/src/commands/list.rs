//! Lists todos using status, project, and result-count filters.

use crate::models::Todo;
use crate::ports::TodoRepository;
use anyhow::Result;

/// Returns todos matching the supplied filters.
pub async fn execute(
    repo: &dyn TodoRepository,
    status: Option<String>,
    project: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<Todo>> {
    repo.list_todos(status.as_deref(), project.as_deref(), limit)
        .await
}
