use crate::db::DbConnection;
use crate::models::{Todo, TodoStatus};
use anyhow::Result;

pub async fn execute(
    db: &DbConnection,
    project: Option<String>,
    status_filter: Option<Vec<TodoStatus>>,
) -> Result<(Vec<Todo>, Option<Vec<TodoStatus>>)> {
    let mut query = String::from("SELECT * FROM todo");

    if let Some(ref p) = project {
        query.push_str(&format!(" WHERE project = '{}'", p));
    }

    query.push_str(" ORDER BY created_at ASC");

    let mut result = db.query(&query).await?;
    let todos: Vec<Todo> = result.take(0)?;

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
