use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use rusqlite::params;

use doob_core::models::handoff_item::{ExtraEntry, HandoffItem};
use doob_core::ports::HandoffRepository;

use crate::db::SqliteConnection;

pub struct HandoffRepositoryImpl {
    db: SqliteConnection,
}

impl HandoffRepositoryImpl {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db }
    }
}

fn row_to_handoff(row: &rusqlite::Row<'_>) -> rusqlite::Result<HandoffItem> {
    let id_val: Option<i64> = row.get("id")?;
    let files_json: String = row.get("files")?;
    let extra_json: String = row.get("extra")?;
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let completed_at_str: Option<String> = row.get("completed_at")?;

    Ok(HandoffItem {
        id: id_val.map(|v| format!("handoff_item:{v}")),
        uuid: row.get("uuid")?,
        handoff_id: row.get("handoff_id")?,
        project: row.get("project")?,
        title: row.get("title")?,
        description: row.get("description")?,
        priority: row.get("priority")?,
        status: row.get("status")?,
        files: serde_json::from_str(&files_json).unwrap_or_default(),
        extra: serde_json::from_str(&extra_json).unwrap_or_default(),
        created_at: chrono::DateTime::parse_from_rfc3339(&normalize_datetime(&created_at_str))
            .unwrap_or_default()
            .with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&normalize_datetime(&updated_at_str))
            .unwrap_or_default()
            .with_timezone(&Utc),
        completed_at: completed_at_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&normalize_datetime(&s))
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        }),
    })
}

fn normalize_datetime(s: &str) -> String {
    if s.contains('T') || s.ends_with('Z') || s.contains('+') {
        s.to_string()
    } else {
        format!("{}Z", s.replace(' ', "T"))
    }
}

#[async_trait]
impl HandoffRepository for HandoffRepositoryImpl {
    async fn get_by_handoff_id(&self, handoff_id: &str) -> Result<Option<HandoffItem>> {
        let hid = handoff_id.to_string();
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, uuid, handoff_id, project, title, description, priority,
                        status, files, extra, created_at, updated_at, completed_at
                 FROM handoff_item WHERE handoff_id = ?1 LIMIT 1",
            )?;
            let item = stmt.query_row(params![hid], row_to_handoff).ok();
            Ok(item)
        })
    }

    async fn list_handoff_items(
        &self,
        project: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<HandoffItem>> {
        self.db.with_conn(|conn| {
            let mut sql = String::from(
                "SELECT id, uuid, handoff_id, project, title, description, priority,
                        status, files, extra, created_at, updated_at, completed_at
                 FROM handoff_item",
            );
            let mut conditions = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(p) = project {
                param_values.push(Box::new(p.to_string()));
                conditions.push(format!("project = ?{}", param_values.len()));
            }
            if let Some(s) = status {
                param_values.push(Box::new(s.to_string()));
                conditions.push(format!("status = ?{}", param_values.len()));
            }
            if !conditions.is_empty() {
                sql.push_str(" WHERE ");
                sql.push_str(&conditions.join(" AND "));
            }
            sql.push_str(" ORDER BY created_at DESC");

            let mut stmt = conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|v| v.as_ref()).collect();
            let rows = stmt
                .query_map(params.as_slice(), row_to_handoff)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        })
    }

    async fn create_handoff_raw(&self, _sql: &str) -> Result<()> {
        // Raw SQL creation is SurrealDB-specific (datetime literal injection).
        // SQLite adapter uses create_handoff_item instead.
        Err(anyhow!(
            "create_handoff_raw is not supported on SQLite; use structured creation"
        ))
    }

    async fn update_handoff_raw(&self, _sql: &str) -> Result<()> {
        Err(anyhow!(
            "update_handoff_raw is not supported on SQLite; use structured updates"
        ))
    }

    async fn update_handoff_status(&self, handoff_id: &str, status: &str) -> Result<()> {
        const VALID: &[&str] = &["open", "done", "parked", "blocked"];
        if !VALID.contains(&status) {
            return Err(anyhow!(
                "Invalid status '{status}'. Valid: {}",
                VALID.join(", ")
            ));
        }

        let now = Utc::now().to_rfc3339();
        let hid = handoff_id.to_string();

        self.db.with_conn(|conn| {
            let exists: bool = conn
                .prepare("SELECT 1 FROM handoff_item WHERE handoff_id = ?1")?
                .exists(params![hid])?;
            if !exists {
                return Err(anyhow!("No handoff item found with id: {handoff_id}"));
            }

            if status == "done" {
                conn.execute(
                    "UPDATE handoff_item SET status = ?1, completed_at = ?2, updated_at = ?2
                     WHERE handoff_id = ?3",
                    params![status, now, hid],
                )?;
            } else {
                conn.execute(
                    "UPDATE handoff_item SET status = ?1, updated_at = ?2
                     WHERE handoff_id = ?3",
                    params![status, now, hid],
                )?;
            }
            Ok(())
        })
    }

    async fn add_extra(&self, handoff_id: &str, entry: ExtraEntry) -> Result<()> {
        let hid = handoff_id.to_string();
        let now = Utc::now().to_rfc3339();

        self.db.with_conn(|conn| {
            let extra_json: String = conn
                .query_row(
                    "SELECT extra FROM handoff_item WHERE handoff_id = ?1",
                    params![hid],
                    |row| row.get(0),
                )
                .map_err(|_| anyhow!("No handoff item found with id: {}", handoff_id))?;

            let mut extra: Vec<ExtraEntry> = serde_json::from_str(&extra_json).unwrap_or_default();
            extra.push(entry);
            let new_json = serde_json::to_string(&extra)?;

            conn.execute(
                "UPDATE handoff_item SET extra = ?1, updated_at = ?2
                 WHERE handoff_id = ?3",
                params![new_json, now, hid],
            )?;
            Ok(())
        })
    }
}
