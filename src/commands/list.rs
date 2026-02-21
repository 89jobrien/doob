use anyhow::Result;
use crate::db::DbConnection;
use crate::models::Todo;

pub async fn execute(
    db: &DbConnection,
    status: Option<String>,
    project: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<Todo>> {
    let mut query = String::from("SELECT * FROM todo");
    let mut conditions = Vec::new();

    if let Some(s) = status {
        conditions.push(format!("status = '{}'", s));
    }

    if let Some(p) = project {
        conditions.push(format!("project = '{}'", p));
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(l) = limit {
        query.push_str(&format!(" LIMIT {}", l));
    }

    let mut result = db.query(&query).await?;
    let todos: Vec<Todo> = result.take(0)?;

    Ok(todos)
}
