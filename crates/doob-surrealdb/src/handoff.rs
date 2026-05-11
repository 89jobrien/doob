use crate::db::DbConnection;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use doob_core::models::handoff_item::{ExtraEntry, HandoffItem};
use doob_core::ports::HandoffRepository;
use doob_core::query_guard::{validate_project, validate_status};

pub struct HandoffRepositoryImpl {
    db: DbConnection,
}

impl HandoffRepositoryImpl {
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl HandoffRepository for HandoffRepositoryImpl {
    async fn get_by_handoff_id(&self, handoff_id: &str) -> Result<Option<HandoffItem>> {
        let hid = handoff_id.replace('\'', "\\'");
        let sql = format!("SELECT * FROM handoff_item WHERE handoff_id = '{hid}' LIMIT 1");
        let mut result = self.db.query(&sql).await?;
        let items: Vec<HandoffItem> = result.take(0).unwrap_or_default();
        Ok(items.into_iter().next())
    }

    async fn list_handoff_items(
        &self,
        project: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<HandoffItem>> {
        let mut query = String::from("SELECT * FROM handoff_item");
        let mut conditions = Vec::new();

        if let Some(p) = project {
            validate_project(p)?;
            conditions.push(format!("project = '{}'", p));
        }

        if let Some(s) = status {
            validate_status(s)?;
            conditions.push(format!("status = '{}'", s));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut result = self.db.query(&query).await?;
        Ok(result.take(0)?)
    }

    async fn create_handoff_raw(&self, sql: &str) -> Result<()> {
        self.db.query(sql).await?;
        Ok(())
    }

    async fn update_handoff_raw(&self, sql: &str) -> Result<()> {
        self.db.query(sql).await?;
        Ok(())
    }

    async fn update_handoff_status(&self, handoff_id: &str, status: &str) -> Result<()> {
        const VALID_STATUSES: &[&str] = &["open", "done", "parked", "blocked"];
        if !VALID_STATUSES.contains(&status) {
            return Err(anyhow!(
                "Invalid status '{}'. Valid: {}",
                status,
                VALID_STATUSES.join(", ")
            ));
        }

        let hid = handoff_id.replace('\'', "\\'");

        // Verify record exists
        let check_sql =
            format!("SELECT handoff_id FROM handoff_item WHERE handoff_id = '{hid}' LIMIT 1");
        let mut check = self.db.query(&check_sql).await?;
        let found: Vec<serde_json::Value> = check.take(0).unwrap_or_default();
        if found.is_empty() {
            return Err(anyhow!("No handoff item found with id: {}", handoff_id));
        }

        let now_str = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
        let completed_set = if status == "done" {
            format!(r#", "completed_at": d"{now_str}""#)
        } else {
            String::new()
        };

        let sql = format!(
            r#"UPDATE handoff_item MERGE {{ "status": "{status}", "updated_at": d"{now_str}"{completed_set} }} WHERE handoff_id = '{hid}'"#
        );
        self.db.query(&sql).await?;
        Ok(())
    }

    async fn add_extra(&self, handoff_id: &str, entry: ExtraEntry) -> Result<()> {
        let item = self
            .get_by_handoff_id(handoff_id)
            .await?
            .ok_or_else(|| anyhow!("No handoff item found with id: {}", handoff_id))?;

        let mut new_extra = item.extra.clone();
        new_extra.push(entry);

        let hid = handoff_id.replace('\'', "\\'");
        let now_str = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
        let extra_json = serde_json::to_string(&new_extra)?;
        let update_sql = format!(
            r#"UPDATE handoff_item MERGE {{ "extra": {extra_json}, "updated_at": d"{now_str}" }} WHERE handoff_id = '{hid}'"#
        );
        self.db.query(&update_sql).await?;
        Ok(())
    }
}
