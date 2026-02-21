use anyhow::{anyhow, Result};
use chrono::Utc;
use crate::db::DbConnection;
use crate::models::{Todo, TodoStatus};

pub async fn execute(db: &DbConnection, ids: Vec<String>) -> Result<usize> {
    let mut completed_count = 0;

    for id in ids {
        // Normalize the record ID format
        let record_id = if id.contains(':') {
            id
        } else {
            format!("todo:{}", id)
        };

        // Get existing todo using query
        let query = format!("SELECT * FROM {} LIMIT 1", record_id);
        let mut result = db.query(&query).await?;
        let todos: Vec<Todo> = result.take(0)?;
        
        if todos.is_empty() {
            return Err(anyhow!("Todo not found: {}", record_id));
        }
        
        let mut todo = todos.into_iter().next().unwrap();

        // Update status and completed_at
        todo.status = TodoStatus::Completed;
        todo.completed_at = Some(Utc::now());
        todo.updated_at = Utc::now();

        // Update using query with explicit values
        let update_query = format!(
            "UPDATE {} SET status = 'completed', completed_at = time::now(), updated_at = time::now()",
            record_id
        );
        db.query(&update_query).await?;
        
        completed_count += 1;
    }

    Ok(completed_count)
}
