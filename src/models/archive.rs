use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedTodo {
    pub id: Option<Thing>,
    pub uuid: String,
    pub content: String,
    pub status: String,
    pub priority: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub project_path: Option<String>,
    pub file_path: Option<String>,
    pub tags: Vec<String>,
    pub archived_at: DateTime<Utc>,
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}
