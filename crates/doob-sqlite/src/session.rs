use anyhow::Result;
use async_trait::async_trait;
use rusqlite::params;

use doob_core::models::handoff::{CommitRef, HandoffState, HandupCheckpoint, LogEntry};
use doob_core::ports::HandoffSessionRepository;

use crate::db::SqliteConnection;

pub struct HandoffSessionRepositoryImpl {
    db: SqliteConnection,
}

impl HandoffSessionRepositoryImpl {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl HandoffSessionRepository for HandoffSessionRepositoryImpl {
    async fn log_append(
        &self,
        project: &str,
        date: &str,
        summary: &str,
        commits: &[String],
    ) -> Result<()> {
        let commits_json = serde_json::to_string(commits)?;
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO handoff_log (project, date, summary, commits)
                 VALUES (?1, ?2, ?3, ?4)",
                params![project, date, summary, commits_json],
            )?;
            Ok(())
        })
    }

    async fn log_query(&self, project: &str) -> Result<Vec<LogEntry>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT date, summary, commits FROM handoff_log
                 WHERE project = ?1 ORDER BY id DESC",
            )?;
            let rows = stmt.query_map(params![project], |row| {
                let date: String = row.get(0)?;
                let summary: String = row.get(1)?;
                let commits_json: String = row.get(2)?;
                Ok((date, summary, commits_json))
            })?;

            let mut entries = Vec::new();
            for row in rows {
                let (date, summary, commits_json) = row?;
                let sha_list: Vec<String> = serde_json::from_str(&commits_json).unwrap_or_default();
                let commits = sha_list.into_iter().map(CommitRef::Sha).collect();
                entries.push(LogEntry {
                    date: Some(date),
                    summary,
                    commits,
                    extra: Default::default(),
                });
            }
            Ok(entries)
        })
    }

    async fn save_state(&self, project: &str, state: &HandoffState) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO handoff_state (project, branch, build, tests, notes, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(project) DO UPDATE SET
                    branch = excluded.branch,
                    build = excluded.build,
                    tests = excluded.tests,
                    notes = excluded.notes,
                    updated_at = excluded.updated_at",
                params![
                    project,
                    state.branch,
                    state.build,
                    state.tests,
                    state.notes,
                    now,
                ],
            )?;
            Ok(())
        })
    }

    async fn load_state(&self, project: &str) -> Result<Option<HandoffState>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT branch, build, tests, notes, updated_at
                 FROM handoff_state WHERE project = ?1",
            )?;
            let state = stmt
                .query_row(params![project], |row| {
                    Ok(HandoffState {
                        branch: row.get(0)?,
                        build: row.get(1)?,
                        tests: row.get(2)?,
                        notes: row.get(3)?,
                        updated: row.get(4)?,
                        touched_files: Vec::new(),
                        last_log: None,
                        extra: Default::default(),
                    })
                })
                .ok();
            Ok(state)
        })
    }

    async fn save_checkpoint(&self, checkpoint: &HandupCheckpoint) -> Result<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO metadata (key, value)
                 VALUES ('initialized:' || ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![checkpoint.project, checkpoint.generated],
            )?;
            // Also store full checkpoint in a dedicated spot if needed
            // For now, handup checkpoints go into metadata as JSON
            let json = serde_json::to_string(&serde_json::json!({
                "project": checkpoint.project,
                "cwd": checkpoint.cwd,
                "generated": checkpoint.generated,
                "recommendation": checkpoint.recommendation,
                "json_path": checkpoint.json_path,
            }))?;
            conn.execute(
                "INSERT INTO metadata (key, value)
                 VALUES ('checkpoint:' || ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![checkpoint.project, json],
            )?;
            Ok(())
        })
    }
}
