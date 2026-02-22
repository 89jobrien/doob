use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc};
use crate::db::DbConnection;
use crate::models::Todo;

pub async fn execute(db: &DbConnection, id: String, due_date: Option<String>) -> Result<()> {
    // Normalize the record ID format
    let record_id = if id.contains(':') {
        id
    } else {
        format!("todo:{}", id)
    };

    // Verify the todo exists
    let query = format!("SELECT * FROM {} LIMIT 1", record_id);
    let mut result = db.query(&query).await?;
    let todos: Vec<Todo> = result.take(0)?;

    if todos.is_empty() {
        return Err(anyhow!("Todo not found: {}", record_id));
    }

    // Update the due_date field
    let update_query = if let Some(date_str) = due_date {
        if date_str.to_lowercase() == "clear" {
            // Clear the due date
            format!(
                "UPDATE {} SET due_date = NONE, updated_at = time::now()",
                record_id
            )
        } else {
            // Parse and set the due date
            let parsed_date = parse_date(&date_str)?;
            let formatted_date = parsed_date.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
            format!(
                "UPDATE {} SET due_date = '{}', updated_at = time::now()",
                record_id, formatted_date
            )
        }
    } else {
        // No date provided, clear it
        format!(
            "UPDATE {} SET due_date = NONE, updated_at = time::now()",
            record_id
        )
    };

    db.query(&update_query).await?;

    Ok(())
}

fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    // Try to parse as YYYY-MM-DD format first
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }

    Err(anyhow!(
        "Invalid date format: '{}'. Expected YYYY-MM-DD or 'clear'",
        date_str
    ))
}
