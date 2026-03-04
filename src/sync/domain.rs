// src/sync/domain.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncableTodo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: u8,
    pub status: TodoStatus,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub file_path: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub external_id: String,
    pub external_url: Option<String>,
    pub provider: String,
    pub synced_at: String,
}
