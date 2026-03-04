use crate::context;
use crate::db::DbConnection;
use crate::models::Note;
use anyhow::Result;
use uuid::Uuid;

pub async fn execute(
    db: &DbConnection,
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

    let mut created_notes = Vec::new();

    for text in content {
        let uuid = Uuid::new_v4().to_string();

        let mut query = String::from(
            "CREATE note SET uuid = $uuid, content = $content, tags = $tags",
        );

        if project.is_some() {
            query.push_str(", project = $project");
        }

        if file_path.is_some() {
            query.push_str(", file_path = $file_path");
        }

        let mut qb = db
            .query(&query)
            .bind(("uuid", uuid))
            .bind(("content", text))
            .bind(("tags", tag_list.clone()));

        if let Some(ref proj) = project {
            qb = qb.bind(("project", proj.clone()));
        }

        if let Some(ref fp) = file_path {
            qb = qb.bind(("file_path", fp.clone()));
        }

        let mut result = qb.await?;
        let created: Option<Note> = result.take(0)?;

        if let Some(note) = created {
            created_notes.push(note);
        }
    }

    Ok(created_notes)
}
