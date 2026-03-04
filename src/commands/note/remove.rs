use crate::commands::note::normalize_note_id;
use crate::db::DbConnection;
use crate::models::Note;
use anyhow::{anyhow, Result};

pub async fn execute(db: &DbConnection, ids: Vec<String>) -> Result<usize> {
    let mut removed_count = 0;

    for id in ids {
        let record_id = normalize_note_id(id);

        let query = format!("SELECT * FROM {} LIMIT 1", record_id);
        let mut result = db.query(&query).await?;
        let notes: Vec<Note> = result.take(0)?;

        if notes.is_empty() {
            return Err(anyhow!("Note not found: {}", record_id));
        }

        let delete_query = format!("DELETE {}", record_id);
        db.query(&delete_query).await?;

        removed_count += 1;
    }

    Ok(removed_count)
}
