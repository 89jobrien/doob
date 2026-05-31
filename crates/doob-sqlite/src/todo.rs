use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use rusqlite::params;

use doob_core::models::note::Note;
use doob_core::models::todo::{Todo, TodoStatus};
use doob_core::ports::TodoRepository;

use crate::db::SqliteConnection;

pub struct TodoRepositoryImpl {
    db: SqliteConnection,
}

impl TodoRepositoryImpl {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db }
    }
}

fn parse_status(s: &str) -> TodoStatus {
    match s {
        "in_progress" => TodoStatus::InProgress,
        "completed" => TodoStatus::Completed,
        "cancelled" => TodoStatus::Cancelled,
        _ => TodoStatus::Pending,
    }
}

fn status_str(s: &TodoStatus) -> &'static str {
    s.as_str()
}

fn parse_json_vec(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

fn row_to_todo(row: &rusqlite::Row<'_>) -> rusqlite::Result<Todo> {
    let id_val: Option<i64> = row.get("id")?;
    let status_str: String = row.get("status")?;
    let tags_json: String = row.get("tags")?;
    let blocks_json: String = row.get("blocks")?;
    let blocked_by_json: String = row.get("blocked_by")?;
    let metadata_str: Option<String> = row.get("metadata")?;
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let completed_at_str: Option<String> = row.get("completed_at")?;
    let due_date_str: Option<String> = row.get("due_date")?;

    Ok(Todo {
        id: id_val.map(|v| format!("todo:{v}")),
        uuid: row.get("uuid")?,
        content: row.get("content")?,
        status: parse_status(&status_str),
        priority: row.get("priority")?,
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
        due_date: due_date_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&normalize_datetime(&s))
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        }),
        project: row.get("project")?,
        project_path: row.get("project_path")?,
        file_path: row.get("file_path")?,
        tags: parse_json_vec(&tags_json),
        metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
        blocks: parse_json_vec(&blocks_json),
        blocked_by: parse_json_vec(&blocked_by_json),
    })
}

/// SQLite datetime('now') produces "YYYY-MM-DD HH:MM:SS" without timezone.
/// Append Z if needed to make it RFC 3339 parseable.
fn normalize_datetime(s: &str) -> String {
    if s.contains('T') || s.ends_with('Z') || s.contains('+') {
        s.to_string()
    } else {
        // "2026-05-11 12:00:00" -> "2026-05-11T12:00:00Z"
        format!("{}Z", s.replace(' ', "T"))
    }
}

fn row_to_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    let id_val: Option<i64> = row.get("id")?;
    let tags_json: String = row.get("tags")?;
    let metadata_str: Option<String> = row.get("metadata")?;
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    Ok(Note {
        id: id_val.map(|v| format!("note:{v}")),
        uuid: row.get("uuid")?,
        content: row.get("content")?,
        created_at: chrono::DateTime::parse_from_rfc3339(&normalize_datetime(&created_at_str))
            .unwrap_or_default()
            .with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&normalize_datetime(&updated_at_str))
            .unwrap_or_default()
            .with_timezone(&Utc),
        project: row.get("project")?,
        file_path: row.get("file_path")?,
        tags: parse_json_vec(&tags_json),
        metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
    })
}

#[async_trait]
impl TodoRepository for TodoRepositoryImpl {
    async fn create_todos(
        &self,
        todos: Vec<(
            String,
            String,
            u8,
            Option<String>,
            Option<String>,
            Vec<String>,
        )>,
    ) -> Result<Vec<Todo>> {
        self.db.with_conn(|conn| {
            let mut result = Vec::new();
            let now = Utc::now().to_rfc3339();
            for (content, uuid, priority, project, file, tags) in todos {
                let tags_json = serde_json::to_string(&tags)?;
                conn.execute(
                    "INSERT INTO todo (uuid, content, priority, project, file_path, tags, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                    params![uuid, content, priority, project, file, tags_json, now],
                )?;
                let id = conn.last_insert_rowid();
                let mut stmt = conn.prepare(
                    "SELECT id, uuid, content, status, priority, created_at, updated_at,
                            completed_at, due_date, project, project_path, file_path,
                            tags, metadata, blocks, blocked_by
                     FROM todo WHERE id = ?1",
                )?;
                let todo = stmt.query_row(params![id], row_to_todo)?;
                result.push(todo);
            }
            Ok(result)
        })
    }

    async fn get_todo(&self, record_id: &str) -> Result<Option<Todo>> {
        let numeric_id = record_id
            .strip_prefix("todo:")
            .unwrap_or(record_id)
            .parse::<i64>()
            .map_err(|_| anyhow!("invalid record id: {record_id}"))?;

        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, uuid, content, status, priority, created_at, updated_at,
                        completed_at, due_date, project, project_path, file_path,
                        tags, metadata, blocks, blocked_by
                 FROM todo WHERE id = ?1",
            )?;
            let todo = stmt.query_row(params![numeric_id], row_to_todo).ok();
            Ok(todo)
        })
    }

    async fn list_todos(
        &self,
        status: Option<&str>,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Todo>> {
        self.db.with_conn(|conn| {
            let mut sql = String::from(
                "SELECT id, uuid, content, status, priority, created_at, updated_at,
                        completed_at, due_date, project, project_path, file_path,
                        tags, metadata, blocks, blocked_by
                 FROM todo",
            );
            let mut conditions = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(s) = status {
                conditions.push(format!("status = ?{}", param_values.len() + 1));
                param_values.push(Box::new(s.to_string()));
            }
            if let Some(p) = project {
                conditions.push(format!("project = ?{}", param_values.len() + 1));
                param_values.push(Box::new(p.to_string()));
            }
            if !conditions.is_empty() {
                sql.push_str(" WHERE ");
                sql.push_str(&conditions.join(" AND "));
            }
            sql.push_str(" ORDER BY priority DESC, created_at ASC");
            if let Some(lim) = limit {
                sql.push_str(&format!(" LIMIT {lim}"));
            }

            let mut stmt = conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|v| v.as_ref()).collect();
            let rows = stmt
                .query_map(params.as_slice(), row_to_todo)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        })
    }

    async fn update_todo(
        &self,
        record_id: &str,
        priority: Option<u8>,
        status: Option<&str>,
        project: Option<&str>,
        tags: Option<Vec<String>>,
        content: Option<&str>,
    ) -> Result<Todo> {
        let numeric_id = record_id
            .strip_prefix("todo:")
            .unwrap_or(record_id)
            .parse::<i64>()
            .map_err(|_| anyhow!("invalid record id: {record_id}"))?;
        let now = Utc::now().to_rfc3339();

        self.db.with_conn(|conn| {
            let mut sets = vec!["updated_at = ?1".to_string()];
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
                vec![Box::new(now.clone())];

            if let Some(p) = priority {
                param_values.push(Box::new(p));
                sets.push(format!("priority = ?{}", param_values.len()));
            }
            if let Some(s) = status {
                param_values.push(Box::new(s.to_string()));
                sets.push(format!("status = ?{}", param_values.len()));
            }
            if let Some(p) = project {
                param_values.push(Box::new(p.to_string()));
                sets.push(format!("project = ?{}", param_values.len()));
            }
            if let Some(t) = tags {
                param_values.push(Box::new(serde_json::to_string(&t)?));
                sets.push(format!("tags = ?{}", param_values.len()));
            }
            if let Some(c) = content {
                param_values.push(Box::new(c.to_string()));
                sets.push(format!("content = ?{}", param_values.len()));
            }

            param_values.push(Box::new(numeric_id));
            let id_param = param_values.len();
            let sql = format!("UPDATE todo SET {} WHERE id = ?{id_param}", sets.join(", "));

            let params: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|v| v.as_ref()).collect();
            conn.execute(&sql, params.as_slice())?;

            let mut stmt = conn.prepare(
                "SELECT id, uuid, content, status, priority, created_at, updated_at,
                        completed_at, due_date, project, project_path, file_path,
                        tags, metadata, blocks, blocked_by
                 FROM todo WHERE id = ?1",
            )?;
            let todo = stmt.query_row(params![numeric_id], row_to_todo)?;
            Ok(todo)
        })
    }

    async fn delete_todo(&self, record_id: &str) -> Result<()> {
        let numeric_id = record_id
            .strip_prefix("todo:")
            .unwrap_or(record_id)
            .parse::<i64>()
            .map_err(|_| anyhow!("invalid record id: {record_id}"))?;

        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM todo WHERE id = ?1", params![numeric_id])?;
            Ok(())
        })
    }

    async fn complete_todo(&self, record_id: &str) -> Result<()> {
        let numeric_id = record_id
            .strip_prefix("todo:")
            .unwrap_or(record_id)
            .parse::<i64>()
            .map_err(|_| anyhow!("invalid record id: {record_id}"))?;
        let now = Utc::now().to_rfc3339();

        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE todo SET status = 'completed', completed_at = ?1, updated_at = ?1
                 WHERE id = ?2",
                params![now, numeric_id],
            )?;
            Ok(())
        })
    }

    async fn undo_todo(&self, record_id: &str) -> Result<()> {
        let numeric_id = record_id
            .strip_prefix("todo:")
            .unwrap_or(record_id)
            .parse::<i64>()
            .map_err(|_| anyhow!("invalid record id: {record_id}"))?;
        let now = Utc::now().to_rfc3339();

        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE todo SET status = 'pending', completed_at = NULL, updated_at = ?1
                 WHERE id = ?2",
                params![now, numeric_id],
            )?;
            Ok(())
        })
    }

    async fn search_todos(&self, query: &str, project: Option<&str>) -> Result<Vec<Todo>> {
        self.db.with_conn(|conn| {
            let pattern = format!("%{query}%");
            let sql = if project.is_some() {
                "SELECT id, uuid, content, status, priority, created_at, updated_at,
                        completed_at, due_date, project, project_path, file_path,
                        tags, metadata, blocks, blocked_by
                 FROM todo WHERE content LIKE ?1 AND project = ?2
                 ORDER BY priority DESC"
            } else {
                "SELECT id, uuid, content, status, priority, created_at, updated_at,
                        completed_at, due_date, project, project_path, file_path,
                        tags, metadata, blocks, blocked_by
                 FROM todo WHERE content LIKE ?1
                 ORDER BY priority DESC"
            };

            let mut stmt = conn.prepare(sql)?;
            let rows: Vec<Todo> = if let Some(p) = project {
                stmt.query_map(params![pattern, p], row_to_todo)?
                    .filter_map(|r| r.ok())
                    .collect()
            } else {
                stmt.query_map(params![pattern], row_to_todo)?
                    .filter_map(|r| r.ok())
                    .collect()
            };
            Ok(rows)
        })
    }

    async fn get_todo_stats(&self) -> Result<serde_json::Value> {
        self.db.with_conn(|conn| {
            let mut stmt =
                conn.prepare("SELECT status, COUNT(*) as cnt FROM todo GROUP BY status")?;
            let rows: Vec<(String, i64)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .filter_map(|r| r.ok())
                .collect();

            let mut map = serde_json::Map::new();
            for (status, count) in rows {
                map.insert(status, serde_json::Value::from(count));
            }
            Ok(serde_json::Value::Object(map))
        })
    }

    async fn set_due_date(&self, record_id: &str, due_date: Option<&str>) -> Result<()> {
        let numeric_id = record_id
            .strip_prefix("todo:")
            .unwrap_or(record_id)
            .parse::<i64>()
            .map_err(|_| anyhow!("invalid record id: {record_id}"))?;
        let now = Utc::now().to_rfc3339();

        self.db.with_conn(|conn| {
            let due = due_date.filter(|d| d.to_lowercase() != "clear");
            conn.execute(
                "UPDATE todo SET due_date = ?1, updated_at = ?2 WHERE id = ?3",
                params![due, now, numeric_id],
            )?;
            Ok(())
        })
    }

    async fn link_deps(&self, uuid: &str, blocks: &[String], blocked_by: &[String]) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let blocks_json = serde_json::to_string(blocks)?;
        let blocked_by_json = serde_json::to_string(blocked_by)?;

        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE todo SET blocks = ?1, blocked_by = ?2, updated_at = ?3
                 WHERE uuid = ?4",
                params![blocks_json, blocked_by_json, now, uuid],
            )?;
            Ok(())
        })
    }

    async fn get_todo_by_uuid(&self, uuid: &str) -> Result<Option<Todo>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, uuid, content, status, priority, created_at, updated_at,
                        completed_at, due_date, project, project_path, file_path,
                        tags, metadata, blocks, blocked_by
                 FROM todo WHERE uuid = ?1",
            )?;
            let todo = stmt.query_row(params![uuid], row_to_todo).ok();
            Ok(todo)
        })
    }

    async fn get_todos_by_uuids(&self, uuids: &[String]) -> Result<Vec<Todo>> {
        if uuids.is_empty() {
            return Ok(Vec::new());
        }
        self.db.with_conn(|conn| {
            let placeholders = std::iter::repeat_n("?", uuids.len())
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!(
                "SELECT id, uuid, content, status, priority, created_at, updated_at,
                        completed_at, due_date, project, project_path, file_path,
                        tags, metadata, blocks, blocked_by
                 FROM todo WHERE uuid IN ({placeholders})"
            );
            let mut stmt = conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::types::ToSql> = uuids
                .iter()
                .map(|u| u as &dyn rusqlite::types::ToSql)
                .collect();
            let rows = stmt
                .query_map(params.as_slice(), row_to_todo)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        })
    }

    async fn list_all_todos(&self, project: Option<&str>) -> Result<Vec<Todo>> {
        self.db.with_conn(|conn| {
            let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) =
                if let Some(p) = project {
                    (
                        "SELECT id, uuid, content, status, priority, created_at, updated_at,
                            completed_at, due_date, project, project_path, file_path,
                            tags, metadata, blocks, blocked_by
                     FROM todo WHERE project = ?1 ORDER BY created_at ASC",
                        vec![Box::new(p.to_string())],
                    )
                } else {
                    (
                        "SELECT id, uuid, content, status, priority, created_at, updated_at,
                            completed_at, due_date, project, project_path, file_path,
                            tags, metadata, blocks, blocked_by
                     FROM todo ORDER BY created_at ASC",
                        vec![],
                    )
                };
            let mut stmt = conn.prepare(sql)?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|v| v.as_ref()).collect();
            let rows = stmt
                .query_map(param_refs.as_slice(), row_to_todo)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        })
    }

    async fn list_active_todos(&self) -> Result<Vec<Todo>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, uuid, content, status, priority, created_at, updated_at,
                        completed_at, due_date, project, project_path, file_path,
                        tags, metadata, blocks, blocked_by
                 FROM todo WHERE status IN ('pending', 'in_progress')
                 ORDER BY priority DESC, created_at ASC",
            )?;
            let rows = stmt
                .query_map([], row_to_todo)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        })
    }

    async fn create_notes(
        &self,
        notes: Vec<(String, Option<String>, Option<String>, Vec<String>)>,
    ) -> Result<Vec<Note>> {
        self.db.with_conn(|conn| {
            let mut result = Vec::new();
            let now = Utc::now().to_rfc3339();
            for (content, project, file, tags) in notes {
                let uuid = uuid::Uuid::new_v4().to_string();
                let tags_json = serde_json::to_string(&tags)?;
                conn.execute(
                    "INSERT INTO note (uuid, content, project, file_path, tags, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                    params![uuid, content, project, file, tags_json, now],
                )?;
                let id = conn.last_insert_rowid();
                let mut stmt = conn.prepare(
                    "SELECT id, uuid, content, created_at, updated_at, project, file_path,
                            tags, metadata
                     FROM note WHERE id = ?1",
                )?;
                let note = stmt.query_row(params![id], row_to_note)?;
                result.push(note);
            }
            Ok(result)
        })
    }

    async fn get_note(&self, record_id: &str) -> Result<Option<Note>> {
        let numeric_id = record_id
            .strip_prefix("note:")
            .unwrap_or(record_id)
            .parse::<i64>()
            .map_err(|_| anyhow!("invalid record id: {record_id}"))?;

        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, uuid, content, created_at, updated_at, project, file_path,
                        tags, metadata
                 FROM note WHERE id = ?1",
            )?;
            let note = stmt.query_row(params![numeric_id], row_to_note).ok();
            Ok(note)
        })
    }

    async fn list_notes(&self, project: Option<&str>, limit: Option<usize>) -> Result<Vec<Note>> {
        self.db.with_conn(|conn| {
            let mut sql = String::from(
                "SELECT id, uuid, content, created_at, updated_at, project, file_path,
                        tags, metadata
                 FROM note",
            );
            if project.is_some() {
                sql.push_str(" WHERE project = ?1");
            }
            sql.push_str(" ORDER BY created_at DESC");
            if let Some(lim) = limit {
                sql.push_str(&format!(" LIMIT {lim}"));
            }

            let mut stmt = conn.prepare(&sql)?;
            let rows: Vec<Note> = if let Some(p) = project {
                stmt.query_map(params![p], row_to_note)?
                    .filter_map(|r| r.ok())
                    .collect()
            } else {
                stmt.query_map([], row_to_note)?
                    .filter_map(|r| r.ok())
                    .collect()
            };
            Ok(rows)
        })
    }

    async fn delete_note(&self, record_id: &str) -> Result<()> {
        let numeric_id = record_id
            .strip_prefix("note:")
            .unwrap_or(record_id)
            .parse::<i64>()
            .map_err(|_| anyhow!("invalid record id: {record_id}"))?;

        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM note WHERE id = ?1", params![numeric_id])?;
            Ok(())
        })
    }

    async fn search_notes(&self, query: &str, project: Option<&str>) -> Result<Vec<Note>> {
        self.db.with_conn(|conn| {
            let pattern = format!("%{query}%");
            let sql = if project.is_some() {
                "SELECT id, uuid, content, created_at, updated_at, project, file_path,
                        tags, metadata
                 FROM note WHERE content LIKE ?1 AND project = ?2"
            } else {
                "SELECT id, uuid, content, created_at, updated_at, project, file_path,
                        tags, metadata
                 FROM note WHERE content LIKE ?1"
            };

            let mut stmt = conn.prepare(sql)?;
            let rows: Vec<Note> = if let Some(p) = project {
                stmt.query_map(params![pattern, p], row_to_note)?
                    .filter_map(|r| r.ok())
                    .collect()
            } else {
                stmt.query_map(params![pattern], row_to_note)?
                    .filter_map(|r| r.ok())
                    .collect()
            };
            Ok(rows)
        })
    }

    async fn execute_raw_query(&self, _query: &str) -> Result<serde_json::Value> {
        // Raw queries are SurrealDB-specific; SQLite adapter doesn't support them
        Err(anyhow!(
            "execute_raw_query is not supported on the SQLite backend"
        ))
    }
}

#[allow(dead_code)]
fn _status_str_ref(s: &TodoStatus) -> &'static str {
    status_str(s)
}
