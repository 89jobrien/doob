//! Serializes note lists as JSON.

use crate::models::Note;
use serde_json::json;

/// Serializes notes as a JSON array.
pub fn format_notes(notes: &[Note]) -> String {
    let output = json!({
        "count": notes.len(),
        "notes": notes
    });

    serde_json::to_string_pretty(&output).unwrap()
}
