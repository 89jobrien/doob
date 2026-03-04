use crate::commands::normalize_id;
use crate::db::DbConnection;
use crate::models::Todo;
use anyhow::{anyhow, Result};

pub async fn execute(db: &DbConnection, ids: Vec<String>) -> Result<usize> {
    let mut removed_count = 0;

    for id in ids {
        let record_id = normalize_id(id);

        // Verify the todo exists before deleting
        let query = format!("SELECT * FROM {} LIMIT 1", record_id);
        let mut result = db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;

        if todos.is_empty() {
            return Err(anyhow!("Todo not found: {}", record_id));
        }

        // Delete the todo
        let delete_query = format!("DELETE {}", record_id);
        db.query(&delete_query).await?;

        removed_count += 1;
    }

    Ok(removed_count)
}
