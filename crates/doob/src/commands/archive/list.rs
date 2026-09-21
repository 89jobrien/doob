//! Lists archived todos with optional project and result limits.

use crate::models::ArchivedTodo;
use crate::ports::ArchiveRepository;
use anyhow::Result;

/// Returns archived todos matching the supplied filters.
pub async fn execute(
    repo: &dyn ArchiveRepository,
    project: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<ArchivedTodo>> {
    repo.list_archived(project.as_deref(), limit).await
}
