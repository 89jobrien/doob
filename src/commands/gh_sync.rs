use crate::db::DbConnection;
use crate::gh_sync;
use crate::models::TodoStatus;
use anyhow::Result;

pub struct GhSyncOptions {
    pub uuid: Option<String>,
    pub dry_run: bool,
    pub force: bool,
    pub action: Option<String>,
}

pub async fn execute(db: &DbConnection, opts: GhSyncOptions) -> Result<()> {
    if let Some(uuid) = opts.uuid {
        // Single-todo sync by UUID
        // Validate UUID format before interpolation — UUIDs are [0-9a-f-] only
        if !uuid.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
            eprintln!("gh-sync: invalid UUID format: {}", uuid);
            return Ok(());
        }
        let query = format!("SELECT * FROM todo WHERE uuid = '{}' LIMIT 1", uuid);
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
        // Bulk sync — query status based on action hint
        let cfg = match crate::gh_sync::config::load()? {
            Some(c) => c,
            None => {
                eprintln!("gh-sync: no config at ~/.config/doob/gh-sync.toml");
                return Ok(());
            }
        };

        let action_hint = opts.action.as_deref().unwrap_or("add");

        let status_filter = match action_hint {
            "complete" => "completed",
            "remove" => "cancelled",
            _ => "pending",
        };

        let query = format!("SELECT * FROM todo WHERE status = '{}'", status_filter);
        let mut result = db.query(&query).await?;
        let todos: Vec<crate::models::Todo> = result.take(0)?;

        let state = crate::gh_sync::state::load()?;

        for todo in todos {
            let project = match &todo.project {
                Some(p) => p.clone(),
                None => continue,
            };
            if crate::gh_sync::mapper::resolve(&project, &cfg).is_none() {
                continue;
            }
            match action_hint {
                "add" => {
                    // --force only applies to "add" — re-closing or re-tombstoning is not supported
                    if !opts.force && crate::gh_sync::state::has_issue(&state, &todo.uuid) {
                        continue;
                    }
                }
                _ => {
                    if !crate::gh_sync::state::has_issue(&state, &todo.uuid) {
                        continue;
                    }
                }
            }
            gh_sync::sync_todo(&todo, action_hint, opts.dry_run)?;
        }
    }

    Ok(())
}
