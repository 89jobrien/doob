use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HandoffItem {
    #[serde(default, deserialize_with = "super::deserialize_thing_to_string")]
    pub id: Option<String>,
    pub uuid: String,
    pub handoff_id: String,
    pub project: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub status: String,
    pub files: Vec<String>,
    pub extra: Vec<ExtraEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtraEntry {
    pub date: String,
    #[serde(rename = "type")]
    pub entry_type: ExtraType,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExtraType {
    Note,
    Blocker,
    Decision,
    Discovery,
    Escalation,
}
