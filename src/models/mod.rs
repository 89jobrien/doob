pub mod archive;
pub mod handoff_item;
pub mod note;
pub mod todo;
pub use archive::ArchivedTodo;
pub use note::Note;
pub use todo::{Todo, TodoStatus};

/// Deserializes SurrealDB's `Thing` type into `Option<String>`.
/// Uses `Thing` for deserialization, then converts to `"table:id"` string.
fn deserialize_thing_to_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let thing: Option<surrealdb::sql::Thing> = Option::deserialize(deserializer)?;
    Ok(thing.map(|t| t.to_string()))
}

use serde::Deserialize;
