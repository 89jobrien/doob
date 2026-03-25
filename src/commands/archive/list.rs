use crate::db::DbConnection;
use crate::models::ArchivedTodo;
use anyhow::Result;

pub async fn execute(
    db: &DbConnection,
    project: Option<String>,
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

    let mut builder = db.query(&query);
    if let Some(ref p) = project {
        builder = builder.bind(("project", p.clone()));
    }

    let mut result = builder.await?;
    Ok(result.take(0)?)
}
