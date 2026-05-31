use crate::commands::search::SearchResults;
use serde_json::json;

pub fn format_results(results: &SearchResults, query: &str) -> String {
    let total = results.todos.len() + results.notes.len();
    let output = json!({
        "query": query,
        "total": total,
        "todos": results.todos,
        "notes": results.notes,
    });
    serde_json::to_string_pretty(&output).unwrap_or_default()
}
