pub mod add;
pub mod list;
pub mod remove;

/// Normalize a note ID to the `note:<id>` record format.
pub fn normalize_note_id(id: String) -> String {
    if id.contains(':') {
        id
    } else {
        format!("note:{}", id)
    }
}
