use crate::models::handoff_item::{ExtraEntry, ExtraType};
use crate::ports::HandoffRepository;
use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

/// Represents one item as stored in HANDOFF.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlItem {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doob_uuid: Option<String>,
    pub name: Option<String>,
    pub priority: String,
    pub status: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub extra: Vec<YamlExtra>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlExtra {
    pub date: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub note: String,
}

/// Top-level HANDOFF.yaml structure -- supports either a bare list or a map with a key
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum HandoffYaml {
    List(Vec<YamlItem>),
    Map(std::collections::HashMap<String, serde_yaml::Value>),
}

#[derive(Debug, Default)]
pub struct SyncSummary {
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub pulled: Vec<String>,
}

/// Payload used for CREATE -- omits datetime fields to avoid SurrealDB JSON coercion issues.
/// Datetimes are set via raw SQL literals in the query string instead.
#[derive(Debug, Serialize, Deserialize)]
struct CreatePayload {
    pub uuid: String,
    pub handoff_id: String,
    pub project: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub priority: String,
    pub status: String,
    pub files: Vec<String>,
    pub extra: Vec<ExtraEntry>,
}

/// Payload used for UPDATE -- omits datetime fields for the same reason.
#[derive(Debug, Serialize, Deserialize)]
struct UpdatePayload {
    pub status: String,
    pub extra: Vec<ExtraEntry>,
}

// qual:allow(iosp) reason: "command handler — file read + DB sync"
pub async fn execute(repo: &dyn HandoffRepository, file: &Path) -> Result<SyncSummary> {
    let raw = std::fs::read_to_string(file).with_context(|| format!("Cannot read {:?}", file))?;

    // Parse: support both bare list and map with "items" key
    let yaml_items: Vec<YamlItem> = {
        let val: serde_yaml::Value = serde_yaml::from_str(&raw)?;
        if val.is_sequence() {
            serde_yaml::from_value(val)?
        } else if let Some(items) = val.get("items") {
            serde_yaml::from_value(items.clone())?
        } else {
            anyhow::bail!("HANDOFF.yaml must be a list or contain an 'items' key");
        }
    };

    // Derive project name from file path (parent dir name)
    let project = file
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut summary = SyncSummary::default();
    let mut updated_yaml_items = yaml_items.clone();

    for (idx, yaml_item) in yaml_items.iter().enumerate() {
        let existing = repo.get_by_handoff_id(&yaml_item.id).await?;

        match existing {
            None => {
                let new_uuid = Uuid::new_v4().to_string();
                let now_str = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
                let completed_at_lit = yaml_item
                    .completed
                    .as_ref()
                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                    .map(|nd| format!("d\"{}T00:00:00.000000Z\"", nd.format("%Y-%m-%d")))
                    .unwrap_or_else(|| "NONE".to_string());

                let extra: Vec<ExtraEntry> = yaml_item
                    .extra
                    .iter()
                    .map(yaml_extra_to_entry)
                    .collect::<Result<Vec<_>>>()?;

                let payload = CreatePayload {
                    uuid: new_uuid.clone(),
                    handoff_id: yaml_item.id.clone(),
                    project: project.clone(),
                    title: yaml_item.title.clone(),
                    description: yaml_item.description.clone(),
                    priority: yaml_item.priority.clone(),
                    status: yaml_item.status.clone(),
                    files: yaml_item.files.clone(),
                    extra,
                };

                let payload_json = serde_json::to_string(&payload)
                    .with_context(|| "Failed to serialize handoff_item payload")?;
                let payload_json = payload_json.trim_end_matches('}');
                let sql = format!(
                    r#"CREATE handoff_item CONTENT {payload_json}, "created_at": d"{now_str}", "updated_at": d"{now_str}", "completed_at": {completed_at_lit} }}"#
                );
                repo.create_handoff_raw(&sql).await.map_err(|e| {
                    anyhow::anyhow!("CREATE handoff_item failed for {}: {e}", yaml_item.id)
                })?;

                updated_yaml_items[idx].doob_uuid = Some(new_uuid.clone());
                summary.created.push(yaml_item.id.clone());
            }
            Some(doob) => {
                // Merge extra: append yaml extras not already in doob (dedup by entry_type+date+note)
                let existing_keys: HashSet<String> =
                    doob.extra.iter().map(extra_entry_dedup_key).collect();

                let new_entries: Vec<ExtraEntry> = yaml_item
                    .extra
                    .iter()
                    .filter(|e| !existing_keys.contains(&yaml_extra_dedup_key(e)))
                    .map(yaml_extra_to_entry)
                    .collect::<Result<Vec<_>>>()?;

                let mut merged_extra = doob.extra.clone();
                merged_extra.extend(new_entries);

                // doob wins on status conflict
                let final_status = doob.status.clone();

                // UPDATE via raw SQL with inline JSON + datetime literals.
                let now_str = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
                let update_payload = UpdatePayload {
                    status: final_status,
                    extra: merged_extra.clone(),
                };
                let update_json = serde_json::to_string(&update_payload)
                    .with_context(|| "Failed to serialize update payload")?;
                let update_json = update_json.trim_end_matches('}');
                let hid_escaped = yaml_item.id.replace('\'', "\\'");
                let update_sql = format!(
                    r#"UPDATE handoff_item MERGE {update_json}, "updated_at": d"{now_str}" }} WHERE handoff_id = '{hid_escaped}'"#
                );
                repo.update_handoff_raw(&update_sql)
                    .await
                    .with_context(|| format!("UPDATE handoff_item failed for {}", yaml_item.id))?;

                // Pull doob status back to yaml if different
                if doob.status != yaml_item.status {
                    updated_yaml_items[idx].status = doob.status.clone();
                    summary.pulled.push(yaml_item.id.clone());
                }

                // Sync doob_uuid back
                updated_yaml_items[idx].doob_uuid = Some(doob.uuid.clone());

                // Merge extra back to yaml
                let yaml_keys: HashSet<String> =
                    yaml_item.extra.iter().map(yaml_extra_dedup_key).collect();
                let doob_only: Vec<YamlExtra> = doob
                    .extra
                    .iter()
                    .filter(|e| !yaml_keys.contains(&extra_entry_dedup_key(e)))
                    .map(entry_to_yaml_extra)
                    .collect();
                if !doob_only.is_empty() {
                    updated_yaml_items[idx].extra.extend(doob_only);
                }

                summary.updated.push(yaml_item.id.clone());
            }
        }
    }

    // Write updated HANDOFF.yaml back -- preserve full document structure.
    let original_val: serde_yaml::Value = serde_yaml::from_str(&raw)?;
    let items_val = serde_yaml::to_value(&updated_yaml_items)?;
    let out = if original_val.is_sequence() {
        serde_yaml::to_string(&items_val)?
    } else {
        let mut doc = original_val;
        if let serde_yaml::Value::Mapping(ref mut m) = doc {
            m.insert(serde_yaml::Value::String("items".to_string()), items_val);
        }
        serde_yaml::to_string(&doc)?
    };
    std::fs::write(file, out)?;

    Ok(summary)
}

fn yaml_extra_to_entry(e: &YamlExtra) -> Result<ExtraEntry> {
    let entry_type = match e.entry_type.as_str() {
        "note" => ExtraType::Note,
        "blocker" => ExtraType::Blocker,
        "decision" => ExtraType::Decision,
        "discovery" => ExtraType::Discovery,
        "escalation" => ExtraType::Escalation,
        other => anyhow::bail!("Unknown extra type: {}", other),
    };
    Ok(ExtraEntry {
        date: e.date.clone(),
        entry_type,
        note: e.note.clone(),
    })
}

fn yaml_extra_dedup_key(e: &YamlExtra) -> String {
    format!("{}|{}|{}", e.entry_type, e.date, e.note)
}

fn extra_entry_dedup_key(e: &ExtraEntry) -> String {
    let et = serde_json::to_string(&e.entry_type)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string();
    format!("{}|{}|{}", et, e.date, e.note)
}

fn entry_to_yaml_extra(e: &ExtraEntry) -> YamlExtra {
    let entry_type = match e.entry_type {
        ExtraType::Note => "note",
        ExtraType::Blocker => "blocker",
        ExtraType::Decision => "decision",
        ExtraType::Discovery => "discovery",
        ExtraType::Escalation => "escalation",
    }
    .to_string();
    YamlExtra {
        date: e.date.clone(),
        entry_type,
        note: e.note.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for doob-8: yaml-side extra dedup must include entry_type in key.
    /// Two extras with same date+note but different types must both survive dedup.
    #[test]
    fn yaml_to_doob_dedup_preserves_different_entry_types_same_date_note() {
        let yaml_extras = vec![
            YamlExtra {
                date: "2026-05-10".to_string(),
                entry_type: "note".to_string(),
                note: "same text".to_string(),
            },
            YamlExtra {
                date: "2026-05-10".to_string(),
                entry_type: "blocker".to_string(),
                note: "same text".to_string(),
            },
        ];

        // Use yaml_extra_dedup_key (was previously just date|note, missing entry_type)
        let yaml_keys: HashSet<String> = yaml_extras.iter().map(yaml_extra_dedup_key).collect();

        // With entry_type in key, these two entries must produce distinct keys
        assert_eq!(
            yaml_keys.len(),
            2,
            "entries with different types must produce different keys"
        );

        // Simulate doob entries matching both yaml entries
        let doob_entries = vec![
            ExtraEntry {
                date: "2026-05-10".to_string(),
                entry_type: ExtraType::Note,
                note: "same text".to_string(),
            },
            ExtraEntry {
                date: "2026-05-10".to_string(),
                entry_type: ExtraType::Blocker,
                note: "same text".to_string(),
            },
        ];

        // Both doob entries exist in yaml, so filtering should yield zero doob-only entries
        let doob_only: Vec<&ExtraEntry> = doob_entries
            .iter()
            .filter(|e| !yaml_keys.contains(&extra_entry_dedup_key(e)))
            .collect();

        assert!(
            doob_only.is_empty(),
            "both doob entries should match yaml keys"
        );
    }
}
