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
    let hid = handoff_id.replace('\'', "\\'");
    let select_sql = format!("SELECT * FROM handoff_item WHERE handoff_id = '{hid}' LIMIT 1");
    let mut result = db.query(&select_sql).await?;
    let items: Vec<HandoffItem> = result.take(0).unwrap_or_default();
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

    let now_str = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let extra_json = serde_json::to_string(&new_extra)?;
    let update_sql = format!(
        r#"UPDATE handoff_item MERGE {{ "extra": {extra_json}, "updated_at": d"{now_str}" }} WHERE handoff_id = '{hid}'"#
    );
    db.query(&update_sql).await?;

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
