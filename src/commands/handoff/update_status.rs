use crate::db::DbConnection;
use anyhow::{anyhow, Result};
use chrono::Utc;

const VALID_STATUSES: &[&str] = &["open", "done", "parked", "blocked"];

pub async fn execute(db: &DbConnection, handoff_id: String, status: String) -> Result<()> {
    if !VALID_STATUSES.contains(&status.as_str()) {
        return Err(anyhow!(
            "Invalid status '{}'. Valid: {}",
            status,
            VALID_STATUSES.join(", ")
        ));
    }

    let hid = handoff_id.replace('\'', "\\'");
    let now_str = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();

    let completed_set = if status == "done" {
        format!(r#", "completed_at": d"{now_str}""#)
    } else {
        String::new()
    };

    let sql = format!(
        r#"UPDATE handoff_item MERGE {{ "status": "{status}", "updated_at": d"{now_str}"{completed_set} }} WHERE handoff_id = '{hid}'"#
    );

    // Verify the record exists first, then update.
    let hid_check = handoff_id.replace('\'', "\\'");
    let check_sql = format!("SELECT handoff_id FROM handoff_item WHERE handoff_id = '{hid_check}' LIMIT 1");
    let mut check = db.query(&check_sql).await?;
    let found: Vec<serde_json::Value> = check.take(0).unwrap_or_default();
    if found.is_empty() {
        return Err(anyhow!("No handoff item found with id: {}", handoff_id));
    }

    db.query(&sql).await?;
    Ok(())
}
