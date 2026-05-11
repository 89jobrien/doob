use crate::db::DbConnection;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use doob_core::ids::quote_record_id;
use doob_core::models::todo::TodoStatus;
use doob_core::models::{ArchivedTodo, Todo};
use doob_core::ports::ArchiveRepository;

pub struct ArchiveRepositoryImpl {
    db: DbConnection,
}

impl ArchiveRepositoryImpl {
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ArchiveRepository for ArchiveRepositoryImpl {
    async fn find_archive_candidates(
        &self,
        cutoff_iso: &str,
        project: Option<&str>,
    ) -> Result<Vec<Todo>> {
        let mut query = format!(
            "SELECT * FROM todo WHERE (status = 'completed' OR status = 'cancelled') \
             AND updated_at < <datetime>'{}' ORDER BY updated_at ASC",
            cutoff_iso
        );
        if project.is_some() {
            query = format!(
                "SELECT * FROM todo WHERE (status = 'completed' OR status = 'cancelled') \
                 AND updated_at < <datetime>'{}' AND project = $project ORDER BY updated_at ASC",
                cutoff_iso
            );
        }

        let mut builder = self.db.query(&query);
        if let Some(p) = project {
            builder = builder.bind(("project", p.to_string()));
        }

        let mut result = builder.await?;
        Ok(result.take(0)?)
    }

    async fn archive_todo(&self, todo: &Todo) -> Result<()> {
        let status_str = match todo.status {
            TodoStatus::Completed => "completed",
            TodoStatus::Cancelled => "cancelled",
            _ => return Ok(()),
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

        self.db
            .query(&insert_query)
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
            let delete_query = format!("DELETE {}", quote_record_id(record_id));
            self.db.query(&delete_query).await?.check()?;
        }

        Ok(())
    }

    async fn list_archived(
        &self,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<ArchivedTodo>> {
        let mut query = String::from("SELECT * FROM archive");

        if project.is_some() {
            query.push_str(" WHERE project = $project");
        }

        query.push_str(" ORDER BY archived_at DESC");

        if let Some(l) = limit {
            query.push_str(&format!(" LIMIT {}", l));
        }

        let mut builder = self.db.query(&query);
        if let Some(p) = project {
            builder = builder.bind(("project", p.to_string()));
        }

        let mut result = builder.await?;
        Ok(result.take(0)?)
    }
}
