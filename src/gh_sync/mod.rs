pub mod config;
pub mod github;
pub mod mapper;
pub mod state;

use crate::models::Todo;
use anyhow::Result;
use serde::Serialize;

/// A planned or completed sync action — returned by sync_todo for rendering.
#[derive(Debug, Serialize)]
pub struct SyncPlan {
    pub repo: String,
    pub action: String,
    pub title: String,
    pub uuid: String,
    /// Set after execution; None in dry-run.
    pub issue_number: Option<u64>,
}

/// Build the GitHub issue body for a todo.
pub fn issue_body(todo: &Todo) -> String {
    format!(
        "{}\n\n---\n_Synced from doob — uuid: {}_",
        todo.content, todo.uuid
    )
}

/// Sync a single todo to GitHub. Idempotent — skips if already in state.
/// `action` is "add", "complete", or "remove".
/// Returns `Some(SyncPlan)` if an action was taken or planned, `None` if skipped.
pub fn sync_todo(todo: &Todo, action: &str, dry_run: bool) -> Result<Option<SyncPlan>> {
    let cfg = match config::load()? {
        Some(c) => c,
        None => {
            eprintln!("gh-sync: no config at ~/.config/doob/gh-sync.toml — skipping");
            return Ok(None);
        }
    };

    let project = match &todo.project {
        Some(p) => p.as_str(),
        None => return Ok(None),
    };

    let repo = match mapper::resolve(project, &cfg) {
        Some(r) => r,
        None => return Ok(None),
    };

    let mut state = state::load()?;

    // Check gh is available before any action (skip in dry-run)
    if !dry_run {
        github::check_gh_available()?;
    }

    match action {
        "add" => {
            if state::has_issue(&state, &todo.uuid) {
                return Ok(None);
            }
            if dry_run {
                return Ok(Some(SyncPlan {
                    repo,
                    action: "create".into(),
                    title: todo.content.clone(),
                    uuid: todo.uuid.clone(),
                    issue_number: None,
                }));
            }
            let body = issue_body(todo);
            let issue_number = github::create_issue(&repo, &todo.content, &body)?;
            state::upsert(&mut state, &todo.uuid, &repo, issue_number);
            state::save(&state)?;
            Ok(Some(SyncPlan {
                repo,
                action: "create".into(),
                title: todo.content.clone(),
                uuid: todo.uuid.clone(),
                issue_number: Some(issue_number),
            }))
        }
        "complete" => {
            let entry = match state.get(&todo.uuid) {
                Some(e) => e.clone(),
                None => return Ok(None),
            };
            if !cfg.sync.close_on_complete {
                return Ok(None);
            }
            if dry_run {
                return Ok(Some(SyncPlan {
                    repo: entry.repo,
                    action: "close".into(),
                    title: todo.content.clone(),
                    uuid: todo.uuid.clone(),
                    issue_number: Some(entry.issue_number),
                }));
            }
            github::close_issue(&entry.repo, entry.issue_number)?;
            Ok(Some(SyncPlan {
                repo: entry.repo,
                action: "close".into(),
                title: todo.content.clone(),
                uuid: todo.uuid.clone(),
                issue_number: Some(entry.issue_number),
            }))
        }
        "remove" => {
            let entry = match state.get(&todo.uuid) {
                Some(e) => e.clone(),
                None => return Ok(None),
            };
            if !cfg.sync.tombstone_on_remove {
                return Ok(None);
            }
            if dry_run {
                return Ok(Some(SyncPlan {
                    repo: entry.repo,
                    action: "tombstone".into(),
                    title: todo.content.clone(),
                    uuid: todo.uuid.clone(),
                    issue_number: Some(entry.issue_number),
                }));
            }
            github::add_comment(
                &entry.repo,
                entry.issue_number,
                "This todo was removed from doob without being completed.",
            )?;
            Ok(Some(SyncPlan {
                repo: entry.repo,
                action: "tombstone".into(),
                title: todo.content.clone(),
                uuid: todo.uuid.clone(),
                issue_number: Some(entry.issue_number),
            }))
        }
        _ => {
            eprintln!("gh-sync: unknown action '{}' — skipping", action);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Todo, TodoStatus};
    use chrono::Utc;

    fn fake_todo(uuid: &str, content: &str) -> Todo {
        Todo {
            id: None,
            uuid: uuid.to_string(),
            content: content.to_string(),
            status: TodoStatus::Pending,
            priority: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            due_date: None,
            project: Some("dev/minibox".to_string()),
            project_path: None,
            file_path: None,
            tags: vec![],
            metadata: None,
            blocks: vec![],
            blocked_by: vec![],
        }
    }

    #[test]
    fn issue_body_contains_content_and_uuid() {
        let todo = fake_todo("test-uuid", "Fix the thing");
        let body = issue_body(&todo);
        assert!(body.contains("Fix the thing"));
        assert!(body.contains("test-uuid"));
    }
}
