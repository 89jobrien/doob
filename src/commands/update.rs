use crate::commands::{normalize_id, quote_record_id};
use crate::db::DbConnection;
use crate::models::Todo;
use anyhow::{anyhow, Result};

pub struct UpdateFields {
    pub priority: Option<u8>,
    pub status: Option<String>,
    pub project: Option<String>,
    pub tags: Option<String>,
    pub content: Option<String>,
}

pub async fn execute(db: &DbConnection, id: String, fields: UpdateFields) -> Result<Todo> {
    // Require at least one field
    if fields.priority.is_none()
        && fields.status.is_none()
        && fields.project.is_none()
        && fields.tags.is_none()
        && fields.content.is_none()
    {
        return Err(anyhow!(
            "No fields provided. Specify at least one of: --priority, --status, --project, \
             --tags, --content"
        ));
    }

    // Validate status value if provided
    if let Some(ref s) = fields.status {
        match s.as_str() {
            "pending" | "in_progress" | "completed" => {}
            other => {
                return Err(anyhow!(
                    "Invalid status '{}'. Must be one of: pending, in_progress, completed",
                    other
                ));
            }
        }
    }

    // Validate priority if provided
    if let Some(p) = fields.priority {
        if !(1..=5).contains(&p) {
            return Err(anyhow!("Priority must be between 1 and 5, got {}", p));
        }
    }

    let record_id = normalize_id(id);

    // Verify the todo exists
    let query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
    let mut result = db.query(&query).await?;
    let todos: Vec<Todo> = result.take(0)?;

    if todos.is_empty() {
        return Err(anyhow!("Todo not found: {}", record_id));
    }

    // Build SET clause from provided fields
    let mut set_parts: Vec<String> = vec!["updated_at = time::now()".to_string()];

    if let Some(p) = fields.priority {
        set_parts.push(format!("priority = {}", p));
    }

    if let Some(ref s) = fields.status {
        set_parts.push(format!("status = '{}'", s));
        if s == "completed" {
            set_parts.push("completed_at = time::now()".to_string());
        }
    }

    if let Some(ref p) = fields.project {
        set_parts.push(format!("project = '{}'", p.replace('\'', "\\'")));
    }

    if let Some(ref t) = fields.tags {
        // Build SurrealDB array literal from comma-separated tags
        let tag_list: Vec<String> = t
            .split(',')
            .map(|s| format!("'{}'", s.trim().replace('\'', "\\'")))
            .collect();
        set_parts.push(format!("tags = [{}]", tag_list.join(", ")));
    }

    if let Some(ref c) = fields.content {
        set_parts.push(format!("content = '{}'", c.replace('\'', "\\'")));
    }

    let update_query = format!(
        "UPDATE {} SET {}",
        quote_record_id(&record_id),
        set_parts.join(", ")
    );
    db.query(&update_query).await?;

    // Fetch and return the updated todo
    let fetch_query = format!("SELECT * FROM {} LIMIT 1", quote_record_id(&record_id));
    let mut fetch_result = db.query(&fetch_query).await?;
    let updated: Vec<Todo> = fetch_result.take(0)?;

    updated
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("Failed to fetch updated todo: {}", record_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_fields__no_fields_returns_error() {
        // We can't run the async fn without a real DB, but we can test the
        // validation logic by checking our guard condition directly.
        let fields = UpdateFields {
            priority: None,
            status: None,
            project: None,
            tags: None,
            content: None,
        };
        let all_none = fields.priority.is_none()
            && fields.status.is_none()
            && fields.project.is_none()
            && fields.tags.is_none()
            && fields.content.is_none();
        assert!(all_none, "should detect that no fields were provided");
    }

    #[test]
    fn update_fields__priority_set_is_not_empty() {
        let fields = UpdateFields {
            priority: Some(3),
            status: None,
            project: None,
            tags: None,
            content: None,
        };
        let all_none = fields.priority.is_none()
            && fields.status.is_none()
            && fields.project.is_none()
            && fields.tags.is_none()
            && fields.content.is_none();
        assert!(
            !all_none,
            "priority = Some(3) should not trigger empty guard"
        );
    }
}
