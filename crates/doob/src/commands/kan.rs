use crate::models::{Todo, TodoStatus};
use crate::ports::TodoRepository;
use anyhow::Result;

pub async fn execute(
    repo: &dyn TodoRepository,
    project: Option<String>,
    status_filter: Option<Vec<TodoStatus>>,
) -> Result<(Vec<Todo>, Option<Vec<TodoStatus>>)> {
    let todos = repo.list_all_todos(project.as_deref()).await?;
    Ok((todos, status_filter))
}

/// Parse a comma-delimited status string into TodoStatus variants.
pub fn parse_status(s: &str) -> Option<TodoStatus> {
    match s.trim().to_lowercase().as_str() {
        "pending" => Some(TodoStatus::Pending),
        "in_progress" | "inprogress" => Some(TodoStatus::InProgress),
        "completed" => Some(TodoStatus::Completed),
        "cancelled" => Some(TodoStatus::Cancelled),
        _ => None,
    }
}
