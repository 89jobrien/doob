use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HandoffItem {
    pub id: Option<Thing>,
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
