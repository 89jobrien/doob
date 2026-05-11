pub mod archive;
pub mod handoff_item;
pub mod note;
pub mod todo;
pub use archive::ArchivedTodo;
pub use note::Note;
pub use todo::{Todo, TodoStatus};

use serde::Deserialize;

/// Deserializes the `id` field as a plain `Option<String>`.
/// Backend adapters are responsible for converting their internal ID types
/// (e.g. SurrealDB `Thing`) to strings before constructing model structs.
fn deserialize_thing_to_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}
