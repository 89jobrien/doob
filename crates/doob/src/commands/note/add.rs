use crate::context;
use crate::models::Note;
use crate::ports::TodoRepository;
use anyhow::Result;

pub async fn execute(
    repo: &dyn TodoRepository,
    content: Vec<String>,
    project: Option<String>,
    file_path: Option<String>,
    tags: Option<String>,
) -> Result<Vec<Note>> {
    let tag_list: Vec<String> = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let project = project.or_else(context::detect_project);
    let file_path = file_path.or_else(context::detect_file_path);

    let mut notes_to_create = Vec::new();

    for text in content {
        notes_to_create.push((text, project.clone(), file_path.clone(), tag_list.clone()));
    }

    repo.create_notes(notes_to_create).await
}
