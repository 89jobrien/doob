pub mod config;
pub mod github;
pub mod mapper;
pub mod state;

use crate::models::Todo;
use anyhow::Result;

/// Build the GitHub issue body for a todo.
pub fn issue_body(todo: &Todo) -> String {
    format!(
        "{}\n\n---\n_Synced from doob — uuid: {}_",
        todo.content, todo.uuid
    )
}

/// Sync a single todo to GitHub. Idempotent — skips if already in state.
/// `action` is "add", "complete", or "remove".
pub fn sync_todo(todo: &Todo, action: &str, dry_run: bool) -> Result<()> {
    let cfg = match config::load()? {
        Some(c) => c,
        None => {
            eprintln!("gh-sync: no config at ~/.config/doob/gh-sync.toml — skipping");
            return Ok(());
        }
    };

    let project = match &todo.project {
        Some(p) => p.as_str(),
        None => return Ok(()),
    };

    let repo = match mapper::resolve(project, &cfg) {
        Some(r) => r,
        None => return Ok(()),
    };

    let mut state = state::load()?;

    match action {
        "add" => {
            if state::has_issue(&state, &todo.uuid) {
                return Ok(());
            }
            let body = issue_body(todo);
            if dry_run {
                println!("[dry-run] Would create issue in {}: {}", repo, todo.content);
                return Ok(());
            }
            github::check_gh_available()?;
            let issue_number = github::create_issue(&repo, &todo.content, &body)?;
            state::upsert(&mut state, &todo.uuid, &repo, issue_number);
            state::save(&state)?;
            println!("✓ Created issue #{} in {}", issue_number, repo);
        }
        "complete" => {
            let entry = match state.get(&todo.uuid) {
                Some(e) => e.clone(),
                None => return Ok(()),
            };
            if !cfg.sync.close_on_complete {
                return Ok(());
            }
            if dry_run {
                println!("[dry-run] Would close issue #{} in {}", entry.issue_number, entry.repo);
                return Ok(());
            }
            github::check_gh_available()?;
            github::close_issue(&entry.repo, entry.issue_number)?;
            println!("✓ Closed issue #{} in {}", entry.issue_number, entry.repo);
        }
        "remove" => {
            let entry = match state.get(&todo.uuid) {
                Some(e) => e.clone(),
                None => return Ok(()),
            };
            if !cfg.sync.tombstone_on_remove {
                return Ok(());
            }
            if dry_run {
                println!(
                    "[dry-run] Would tombstone issue #{} in {}",
                    entry.issue_number, entry.repo
                );
                return Ok(());
            }
            github::check_gh_available()?;
            github::add_comment(
                &entry.repo,
                entry.issue_number,
                "This todo was removed from doob without being completed.",
            )?;
            println!("✓ Tombstoned issue #{} in {}", entry.issue_number, entry.repo);
        }
        _ => {
            eprintln!("gh-sync: unknown action '{}' — skipping", action);
        }
    }

    Ok(())
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
