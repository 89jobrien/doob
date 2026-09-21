//! Lists notes with optional project and result limits.

use crate::models::Note;
use crate::ports::TodoRepository;
use anyhow::Result;

/// Returns notes matching the supplied filters.
pub async fn execute(
    repo: &dyn TodoRepository,
    project: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<Note>> {
    repo.list_notes(project.as_deref(), limit).await
}
