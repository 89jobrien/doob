use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: Option<Thing>,
    pub uuid: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub project: Option<String>,
    pub file_path: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}
