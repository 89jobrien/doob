use crate::models::handoff_item::{ExtraEntry, ExtraType};
use crate::ports::HandoffRepository;
use anyhow::{anyhow, Result};
use chrono::Utc;

pub async fn execute(
    repo: &dyn HandoffRepository,
    handoff_id: String,
    entry_type: String,
    note: String,
) -> Result<()> {
    let et = parse_extra_type(&entry_type)?;
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let entry = ExtraEntry {
        date: today,
        entry_type: et,
        note,
    };

    repo.add_extra(&handoff_id, entry).await
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
