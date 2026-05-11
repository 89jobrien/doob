use crate::commands::archive::run::ArchiveRunResult;
use crate::models::ArchivedTodo;

pub fn format_run_result(result: &ArchiveRunResult) -> String {
    if result.dry_run {
        if result.candidates.is_empty() {
            return "No todos eligible for archiving.".to_string();
        }
        let mut out = format!(
            "Would archive {} todo(s) (dry run — use --apply to execute):\n\n",
            result.candidates.len()
        );
        for todo in &result.candidates {
            out.push_str(&format!("  [{}] {}\n", &todo.uuid[..8], todo.content));
            if let Some(ref proj) = todo.project {
                out.push_str(&format!("       project: {}\n", proj));
            }
        }
        out
    } else {
        format!("✓ Archived {} todo(s).\n", result.archived_count)
    }
}

pub fn format_list(todos: &[ArchivedTodo]) -> String {
    if todos.is_empty() {
        return "No archived todos found.".to_string();
    }

    let mut out = String::new();
    for todo in todos {
        let short_id = todo
            .id
            .as_ref()
            .map(|t| t.split(':').next_back().unwrap_or(t).to_string())
            .unwrap_or_else(|| todo.uuid[..8].to_string());

        out.push_str(&format!(
            "  {}  [{}] {}\n",
            short_id, todo.status, todo.content
        ));
        out.push_str(&format!(
            "         archived: {}\n",
            todo.archived_at.format("%Y-%m-%d %H:%M")
        ));
        if let Some(ref proj) = todo.project {
            out.push_str(&format!("         project:  {}\n", proj));
        }
    }
    out
}
