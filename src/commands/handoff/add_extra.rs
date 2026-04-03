use crate::db::DbConnection;
use crate::models::handoff_item::{ExtraEntry, ExtraType, HandoffItem};
use anyhow::{anyhow, Result};
use chrono::Utc;

pub async fn execute(
    db: &DbConnection,
    handoff_id: String,
    entry_type: String,
    note: String,
) -> Result<()> {
    // Look up existing item
    let mut result = db
        .query("SELECT * FROM handoff_item WHERE handoff_id = $id LIMIT 1")
        .bind(("id", handoff_id.clone()))
        .await?;
    let items: Vec<HandoffItem> = result.take(0)?;
    let item = items
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No handoff item found with id: {}", handoff_id))?;

    let et = parse_extra_type(&entry_type)?;
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let entry = ExtraEntry {
        date: today,
        entry_type: et,
        note,
    };

    let mut new_extra = item.extra.clone();
    new_extra.push(entry);

    let now = Utc::now();

    db.query("UPDATE handoff_item SET extra = $extra, updated_at = $now WHERE handoff_id = $id")
        .bind(("extra", serde_json::to_value(&new_extra)?))
        .bind(("now", now))
        .bind(("id", handoff_id))
        .await?;

    Ok(())
}

fn parse_extra_type(s: &str) -> Result<ExtraType> {
    match s {
        "note" => Ok(ExtraType::Note),
        "blocker" => Ok(ExtraType::Blocker),
        "decision" => Ok(ExtraType::Decision),
        "discovery" => Ok(ExtraType::Discovery),
        "escalation" => Ok(ExtraType::Escalation),
        other => Err(anyhow!(
            "Unknown extra type '{}'. Valid: note, blocker, decision, discovery, escalation",
            other
        )),
    }
}
