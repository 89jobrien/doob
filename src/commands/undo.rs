use anyhow::{anyhow, Result};
use crate::db::DbConnection;
use crate::models::{Todo, TodoStatus};

pub async fn execute(db: &DbConnection, ids: Vec<String>) -> Result<usize> {
    let mut undone_count = 0;

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

        let todo = todos.into_iter().next().unwrap();

        // Only allow undo for completed todos
        if todo.status != TodoStatus::Completed {
            return Err(anyhow!(
                "Todo {} is not completed (current status: {:?})",
                record_id,
                todo.status
            ));
        }

        // Update status back to pending
        let update_query = format!(
            "UPDATE {} SET status = 'pending', updated_at = time::now()",
            record_id
        );
        db.query(&update_query).await?;

        undone_count += 1;
    }

    Ok(undone_count)
}
