use crate::context;
use crate::models::Todo;
use crate::ports::TodoRepository;
use anyhow::Result;

pub async fn execute(
    repo: &dyn TodoRepository,
    content: Vec<String>,
    priority: Option<u8>,
    project: Option<String>,
    file_path: Option<String>,
    tags: Option<String>,
) -> Result<Vec<Todo>> {
    let priority = priority.unwrap_or(0);
    let tag_list: Vec<String> = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // Auto-detect context if not provided
    let project = project.or_else(context::detect_project);
    let file_path = file_path.or_else(context::detect_file_path);

    let mut todos_to_create = Vec::new();

    for task in content {
        todos_to_create.push((
            uuid::Uuid::new_v4().to_string(),
            task,
            priority,
            project.clone(),
            file_path.clone(),
            tag_list.clone(),
        ));
    }

    repo.create_todos(todos_to_create).await
}
