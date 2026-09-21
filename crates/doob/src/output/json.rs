//! Serializes todo lists as JSON.

use crate::models::Todo;
use serde_json::json;

/// Serializes todos as a JSON array.
pub fn format_todos(todos: &[Todo]) -> String {
    let output = json!({
        "count": todos.len(),
        "todos": todos
    });

    serde_json::to_string_pretty(&output).unwrap()
}
