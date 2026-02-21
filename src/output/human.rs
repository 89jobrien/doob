use crate::models::{Todo, TodoStatus};

pub fn format_todos(todos: &[Todo]) -> String {
    if todos.is_empty() {
        return "No todos found".to_string();
    }

    let mut output = String::new();
    for (i, todo) in todos.iter().enumerate() {
        output.push_str(&format!(
            "{}. [{}] {} (priority: {})\n",
            i + 1,
            status_str(&todo.status),
            todo.content,
            todo.priority
        ));

        if let Some(proj) = &todo.project {
            output.push_str(&format!("   Project: {}\n", proj));
        }

        if !todo.tags.is_empty() {
            output.push_str(&format!("   Tags: {}\n", todo.tags.join(", ")));
        }
    }

    output
}

fn status_str(status: &TodoStatus) -> &str {
    match status {
        TodoStatus::Pending => "pending",
        TodoStatus::InProgress => "in_progress",
        TodoStatus::Completed => "completed",
        TodoStatus::Cancelled => "cancelled",
    }
}
