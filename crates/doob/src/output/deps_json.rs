//! Serializes todo dependency relationships as JSON.

use crate::commands::deps::DepsView;
use serde_json::json;

/// Serializes a todo with its blockers and dependents.
pub fn format_deps(view: &DepsView) -> String {
    let output = json!({
        "root": view.root,
        "blockers": view.blockers,
        "dependents": view.dependents,
    });
    serde_json::to_string_pretty(&output).unwrap()
}
