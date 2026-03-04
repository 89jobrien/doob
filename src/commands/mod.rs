pub mod add;
pub mod complete;
pub mod due;
pub mod kan;
pub mod list;
pub mod note;
pub mod remove;
pub mod undo;

/// Normalize a todo ID to the `todo:<id>` record format.
pub fn normalize_id(id: String) -> String {
    if id.contains(':') {
        id
    } else {
        format!("todo:{}", id)
    }
}
