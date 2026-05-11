// cache.rs -- writes ~/.cache/doob/status.json after every mutation.
//
// Downstream consumers (Starship, Nu hooks) read this file to display
// overdue counts without shelling out to doob. Cache writes are always
// best-effort: a failure must never fail the CLI command.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;

use crate::models::{Todo, TodoStatus};
use crate::ports::TodoRepository;

#[derive(Serialize)]
pub struct StatusCache {
    pub updated_at: String,
    pub pending_total: usize,
    pub overdue_total: usize,
    pub overdue_by_repo: HashMap<String, usize>,
}

pub fn cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".cache/doob/status.json")
}

pub fn write_status_cache(cache: &StatusCache) -> Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cache)?;
    fs::write(&path, json)?;
    Ok(())
}

pub async fn build_status_cache(repo: &dyn TodoRepository) -> Result<StatusCache> {
    let todos: Vec<Todo> = repo.list_active_todos().await?;

    let now = Utc::now();
    let mut pending_total = 0usize;
    let mut overdue_total = 0usize;
    let mut overdue_by_repo: HashMap<String, usize> = HashMap::new();

    for todo in &todos {
        if matches!(todo.status, TodoStatus::Pending | TodoStatus::InProgress) {
            pending_total += 1;
            if let Some(due) = todo.due_date {
                if due < now {
                    overdue_total += 1;
                    if let Some(ref repo_name) = todo.project {
                        *overdue_by_repo.entry(repo_name.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    Ok(StatusCache {
        updated_at: now.to_rfc3339(),
        pending_total,
        overdue_total,
        overdue_by_repo,
    })
}

/// Rebuild and write the cache. Best-effort -- logs to stderr on failure.
pub async fn refresh(repo: &dyn TodoRepository) {
    match build_status_cache(repo).await {
        Ok(cache) => {
            if let Err(e) = write_status_cache(&cache) {
                eprintln!("[doob cache] write failed: {e}");
            }
        }
        Err(e) => eprintln!("[doob cache] build failed: {e}"),
    }
}
