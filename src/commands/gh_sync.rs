use crate::db::DbConnection;
use crate::gh_sync;
use crate::models::TodoStatus;
use anyhow::Result;

pub struct GhSyncOptions {
    pub uuid: Option<String>,
    pub dry_run: bool,
    pub force: bool,
}

pub async fn execute(db: &DbConnection, opts: GhSyncOptions) -> Result<()> {
    if let Some(uuid) = opts.uuid {
        // Single-todo sync by UUID
        let query = format!(
            "SELECT * FROM todo WHERE uuid = '{}' LIMIT 1",
            uuid.replace('\'', "")
        );
        let mut result = db.query(&query).await?;
        let todos: Vec<crate::models::Todo> = result.take(0)?;
        let todo = match todos.into_iter().next() {
            Some(t) => t,
            None => {
                eprintln!("gh-sync: todo not found for uuid {}", uuid);
                return Ok(());
            }
        };
        let action = match todo.status {
            TodoStatus::Completed => "complete",
            TodoStatus::Cancelled => "remove",
            _ => "add",
        };
        gh_sync::sync_todo(&todo, action, opts.dry_run)?;
    } else {
        // Bulk sync — all pending todos in allowlisted projects not yet in state
        let query = "SELECT * FROM todo WHERE status = 'pending'".to_string();
        let mut result = db.query(&query).await?;
        let todos: Vec<crate::models::Todo> = result.take(0)?;

        let state = crate::gh_sync::state::load()?;
        let cfg = match crate::gh_sync::config::load()? {
            Some(c) => c,
            None => {
                eprintln!("gh-sync: no config at ~/.config/doob/gh-sync.toml");
                return Ok(());
            }
        };

        for todo in todos {
            let project = match &todo.project {
                Some(p) => p.clone(),
                None => continue,
            };
            if crate::gh_sync::mapper::resolve(&project, &cfg).is_none() {
                continue;
            }
            if !opts.force && crate::gh_sync::state::has_issue(&state, &todo.uuid) {
                continue;
            }
            gh_sync::sync_todo(&todo, "add", opts.dry_run)?;
        }
    }

    Ok(())
}
