use anyhow::Result;
use uuid::Uuid;
use crate::db::DbConnection;
use crate::models::Todo;

pub async fn execute(
    db: &DbConnection,
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

    let mut created_todos = Vec::new();

    for task in content {
        let uuid = Uuid::new_v4().to_string();
        
        // Build query dynamically based on optional fields
        let mut query = String::from("CREATE todo SET uuid = $uuid, content = $content, status = 'pending', priority = $priority, tags = $tags");
        
        if project.is_some() {
            query.push_str(", project = $project");
        }
        
        if file_path.is_some() {
            query.push_str(", file_path = $file_path");
        }
        
        let mut query_builder = db.query(&query)
            .bind(("uuid", uuid))
            .bind(("content", task))
            .bind(("priority", priority))
            .bind(("tags", tag_list.clone()));
            
        if let Some(ref proj) = project {
            query_builder = query_builder.bind(("project", proj.clone()));
        }
        
        if let Some(ref fp) = file_path {
            query_builder = query_builder.bind(("file_path", fp.clone()));
        }
        
        let mut result = query_builder.await?;
        let created: Option<Todo> = result.take(0)?;
        
        if let Some(todo) = created {
            created_todos.push(todo);
        }
    }

    Ok(created_todos)
}
