pub mod add;
pub mod list;
pub mod complete;
pub mod remove;
pub mod due;
pub mod undo;

/// Normalize a todo ID to the `todo:<id>` record format.
pub fn normalize_id(id: String) -> String {
    if id.contains(':') {
        id
    } else {
        format!("todo:{}", id)
    }
}
