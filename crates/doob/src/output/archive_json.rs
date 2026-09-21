//! Serializes archive command results as JSON.

use crate::commands::archive::run::ArchiveRunResult;
use crate::models::ArchivedTodo;
use serde_json::json;

/// Serializes archive candidates and application totals.
pub fn format_run_result(result: &ArchiveRunResult) -> String {
    let output = json!({
        "dry_run": result.dry_run,
        "candidates": result.candidates,
        "archived_count": result.archived_count,
    });
    serde_json::to_string_pretty(&output).unwrap()
}

/// Serializes archived todos as a JSON array.
pub fn format_list(todos: &[ArchivedTodo]) -> String {
    let output = json!({
        "count": todos.len(),
        "archived": todos,
    });
    serde_json::to_string_pretty(&output).unwrap()
}
