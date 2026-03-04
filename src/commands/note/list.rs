use crate::db::DbConnection;
use crate::models::Note;
use anyhow::Result;

pub async fn execute(
    db: &DbConnection,
    project: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<Note>> {
    let mut query = String::from("SELECT * FROM note");

    if let Some(ref p) = project {
        query.push_str(&format!(" WHERE project = '{}'", p));
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(l) = limit {
        query.push_str(&format!(" LIMIT {}", l));
    }

    let mut result = db.query(&query).await?;
    let notes: Vec<Note> = result.take(0)?;

    Ok(notes)
}
