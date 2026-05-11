use crate::models::Note;
use crate::ports::TodoRepository;
use anyhow::Result;

pub async fn execute(
    repo: &dyn TodoRepository,
    project: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<Note>> {
    repo.list_notes(project.as_deref(), limit).await
}
