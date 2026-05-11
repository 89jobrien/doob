pub mod archive;
pub mod handoff_item;
pub mod note;
pub mod todo;
pub use archive::ArchivedTodo;
pub use note::Note;
pub use todo::{Todo, TodoStatus};

use serde::Deserialize;

/// Deserializes the `id` field from the active backend.
/// With SurrealDB, converts `Thing { tb, id }` into `"table:id"` strings.
/// Without SurrealDB, passes through as a plain `Option<String>`.
#[cfg(feature = "surrealdb-backend")]
fn deserialize_thing_to_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let thing: Option<surrealdb::sql::Thing> = Option::deserialize(deserializer)?;
    Ok(thing.map(|t| t.to_string()))
}

#[cfg(not(feature = "surrealdb-backend"))]
fn deserialize_thing_to_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}
