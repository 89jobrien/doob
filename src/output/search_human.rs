use crate::commands::search::SearchResults;
use crate::models::TodoStatus;

pub fn format_results(results: &SearchResults) -> String {
    if results.todos.is_empty() && results.notes.is_empty() {
        return format!("No results for \"{}\"", results.query);
    }

    let mut out = String::new();

    out.push_str(&format!("=== Todos ({}) ===\n", results.todos.len()));
    if results.todos.is_empty() {
        out.push_str("  (no results)\n");
    } else {
        for (i, todo) in results.todos.iter().enumerate() {
            out.push_str(&format!(
                "{}. [{}] {}\n",
                i + 1,
                status_str(&todo.status),
                todo.content
            ));
            if let Some(proj) = &todo.project {
                out.push_str(&format!("   Project: {}\n", proj));
            }
            if !todo.tags.is_empty() {
                out.push_str(&format!("   Tags: {}\n", todo.tags.join(", ")));
            }
        }
    }

    out.push('\n');

    out.push_str(&format!("=== Notes ({}) ===\n", results.notes.len()));
    if results.notes.is_empty() {
        out.push_str("  (no results)\n");
    } else {
        for note in &results.notes {
            let short_id = note
                .id
                .as_ref()
                .map(|t| t.id.to_string())
                .unwrap_or_else(|| note.uuid[..8].to_string());
            out.push_str(&format!("  {}  {}\n", short_id, note.content));
            if let Some(proj) = &note.project {
                out.push_str(&format!("         Project: {}\n", proj));
            }
        }
    }

    out
}

fn status_str(status: &TodoStatus) -> &str {
    match status {
        TodoStatus::Pending => "pending",
        TodoStatus::InProgress => "in_progress",
        TodoStatus::Completed => "completed",
        TodoStatus::Cancelled => "cancelled",
    }
}
