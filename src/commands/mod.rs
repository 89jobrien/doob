pub mod add;
pub mod archive;
pub mod handoff;
pub mod complete;
pub mod deps;
pub mod due;
pub mod kan;
pub mod list;
pub mod note;
pub mod remove;
pub mod schema;
pub mod search;
pub mod stats;
pub mod undo;
pub mod watch;

/// Normalize a todo ID to the `todo:<id>` record format.
pub fn normalize_id(id: String) -> String {
    if id.contains(':') {
        id
    } else {
        format!("todo:{}", id)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::normalize_id;

    #[test]
    fn normalize_id__passes_through_namespaced_id() {
        assert_eq!(normalize_id("todo:abc123".to_string()), "todo:abc123");
    }

    #[test]
    fn normalize_id__prefixes_bare_id() {
        assert_eq!(normalize_id("abc123".to_string()), "todo:abc123");
    }

    #[test]
    fn normalize_id__handles_other_namespaces() {
        // Any ID containing ':' passes through unchanged
        assert_eq!(normalize_id("note:xyz".to_string()), "note:xyz");
    }
}
