use crate::commands::quote_record_id;
use crate::db::DbConnection;
use crate::models::{Todo, TodoStatus};
use anyhow::Result;
use chrono::Utc;

pub struct ArchiveRunResult {
    pub dry_run: bool,
    pub candidates: Vec<Todo>,
    pub archived_count: usize,
}

pub async fn execute(
    db: &DbConnection,
    older_than_days: u32,
    apply: bool,
    project: Option<String>,
) -> Result<ArchiveRunResult> {
    let cutoff = Utc::now() - chrono::Duration::days(older_than_days as i64);
    let cutoff_str = cutoff.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let mut query = format!(
        "SELECT * FROM todo WHERE (status = 'completed' OR status = 'cancelled') AND updated_at < <datetime>'{}' ORDER BY updated_at ASC",
        cutoff_str
    );
    if project.is_some() {
        query = format!(
            "SELECT * FROM todo WHERE (status = 'completed' OR status = 'cancelled') AND updated_at < <datetime>'{}' AND project = $project ORDER BY updated_at ASC",
            cutoff_str
        );
    }

    let mut builder = db.query(&query);
    if let Some(ref p) = project {
        builder = builder.bind(("project", p.clone()));
    }

    let mut result = builder.await?;
    let candidates: Vec<Todo> = result.take(0)?;

    if !apply {
        return Ok(ArchiveRunResult {
            dry_run: true,
            candidates,
            archived_count: 0,
        });
    }

    let mut archived_count = 0usize;

    for todo in &candidates {
        let status_str = match todo.status {
            TodoStatus::Completed => "completed",
            TodoStatus::Cancelled => "cancelled",
            _ => continue,
        };

        let fmt = |dt: chrono::DateTime<Utc>| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let created_at_str = fmt(todo.created_at);
        let updated_at_str = fmt(todo.updated_at);
        let completed_at_clause = match todo.completed_at {
            Some(dt) => format!("<datetime>'{}'", fmt(dt)),
            None => "NONE".to_string(),
        };
        let due_date_clause = match todo.due_date {
            Some(dt) => format!("<datetime>'{}'", fmt(dt)),
            None => "NONE".to_string(),
        };

        let insert_query = format!(
            "CREATE archive SET \
            uuid = $uuid, content = $content, status = $status, \
            priority = $priority, \
            created_at = <datetime>'{}', updated_at = <datetime>'{}', \
            completed_at = {}, due_date = {}, \
            project = $project, project_path = $project_path, file_path = $file_path, \
            tags = $tags, blocks = $blocks, blocked_by = $blocked_by",
            created_at_str, updated_at_str, completed_at_clause, due_date_clause
        );

        db.query(&insert_query)
            .bind(("uuid", todo.uuid.clone()))
            .bind(("content", todo.content.clone()))
            .bind(("status", status_str.to_string()))
            .bind(("priority", todo.priority))
            .bind(("project", todo.project.clone()))
            .bind(("project_path", todo.project_path.clone()))
            .bind(("file_path", todo.file_path.clone()))
            .bind(("tags", todo.tags.clone()))
            .bind(("blocks", todo.blocks.clone()))
            .bind(("blocked_by", todo.blocked_by.clone()))
            .await?
            .check()?;

        if let Some(ref record_id) = todo.id {
            let record_id_str = record_id.to_string();
            let delete_query = format!("DELETE {}", quote_record_id(&record_id_str));
            db.query(&delete_query).await?.check()?;
            archived_count += 1;
        }
    }

    Ok(ArchiveRunResult {
        dry_run: false,
        candidates,
        archived_count,
    })
}
