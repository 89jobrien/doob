pub mod add;
pub mod archive;
pub mod complete;
pub mod deps;
pub mod due;
pub mod handoff;
pub mod kan;
pub mod list;
pub mod note;
pub mod remove;
pub mod schema;
pub mod search;
pub mod stats;
pub mod undo;
pub mod update;
pub mod watch;

/// Normalize a todo ID to the `todo:<id>` record format.
pub fn normalize_id(id: String) -> String {
    if id.contains(':') {
        id
    } else {
        format!("todo:{}", id)
    }
}

/// Quote a normalized record ID for use in raw SurrealDB queries.
///
/// SurrealDB parses `todo:<id>` differently depending on the ID format:
/// - Alphanumeric IDs (e.g. `todo:abc123`) — no quoting needed
/// - UUID-format IDs with hyphens (e.g. `todo:006d7f55-3159-...`) — must be
///   backtick-wrapped as `` `todo:006d7f55-3159-...` `` or the parser chokes
///   on the hyphens/letters following numeric segments
pub fn quote_record_id(record_id: &str) -> String {
    let id_part = record_id.split_once(':').map(|x| x.1).unwrap_or(record_id);
    if id_part.contains('-') {
        format!("`{}`", record_id)
    } else {
        record_id.to_string()
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
