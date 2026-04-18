use crate::db::DbConnection;
use crate::models::handoff_item::HandoffItem;
use crate::query_guard::{validate_project, validate_status};
use anyhow::Result;

pub async fn execute(
    db: &DbConnection,
    project: Option<String>,
    status: Option<String>,
) -> Result<Vec<HandoffItem>> {
    let mut query = String::from("SELECT * FROM handoff_item");
    let mut conditions = Vec::new();

    if let Some(p) = project {
        validate_project(&p)?;
        conditions.push(format!("project = '{}'", p));
    }

    if let Some(s) = status {
        validate_status(&s)?;
        conditions.push(format!("status = '{}'", s));
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY created_at DESC");

    let mut result = db.query(&query).await?;

    let items: Vec<HandoffItem> = result.take(0)?;
    Ok(items)
}

// Validation tests live in crate::query_guard
