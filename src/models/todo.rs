use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    #[serde(default, deserialize_with = "super::deserialize_thing_to_string")]
    pub id: Option<String>,
    pub uuid: String,
    pub content: String,
    pub status: TodoStatus,
    pub priority: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub project_path: Option<String>,
    pub file_path: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}
