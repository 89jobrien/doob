use crate::db::DbConnection;
use crate::gh_sync::{self, SyncPlan};
use crate::models::TodoStatus;
use anyhow::Result;
use colored::Colorize;
use std::collections::BTreeMap;

pub struct GhSyncOptions {
    pub uuid: Option<String>,
    pub dry_run: bool,
    pub force: bool,
    pub action: Option<String>,
    pub json: bool,
}

pub async fn execute(db: &DbConnection, opts: GhSyncOptions) -> Result<()> {
    let mut plans: Vec<SyncPlan> = Vec::new();

    if let Some(uuid) = opts.uuid {
        // Single-todo sync by UUID
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
        if let Some(plan) = gh_sync::sync_todo(&todo, action, opts.dry_run)? {
            plans.push(plan);
        }
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
            if let Some(plan) = gh_sync::sync_todo(&todo, action_hint, opts.dry_run)? {
                plans.push(plan);
            }
        }
    }

    render(&plans, opts.dry_run, opts.json);
    Ok(())
}

fn render(plans: &[SyncPlan], dry_run: bool, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(plans).unwrap_or_default());
        return;
    }

    if plans.is_empty() {
        println!("{}", "Nothing to sync.".dimmed());
        return;
    }

    // Group by repo (BTreeMap for stable sort order)
    let mut by_repo: BTreeMap<&str, Vec<&SyncPlan>> = BTreeMap::new();
    for plan in plans {
        by_repo.entry(&plan.repo).or_default().push(plan);
    }

    let header = if dry_run {
        format!("Dry run — {} issue(s) would be affected", plans.len())
    } else {
        format!("{} issue(s) synced", plans.len())
    };
    println!("\n{}\n", header.bold());

    for (repo, items) in &by_repo {
        println!(
            "  {} ({})",
            repo.cyan().bold(),
            items.len().to_string().dimmed()
        );
        for plan in items {
            let (symbol, title_colored) = match plan.action.as_str() {
                "create" => ("+".green().bold(), truncate(&plan.title, 72).green()),
                "close" => ("-".yellow().bold(), truncate(&plan.title, 72).yellow()),
                "tombstone" => ("~".dimmed(), truncate(&plan.title, 72).dimmed()),
                _ => ("+".normal(), truncate(&plan.title, 72).normal()),
            };
            let issue_str = match plan.issue_number {
                Some(n) => format!(" #{}", n).dimmed().to_string(),
                None => String::new(),
            };
            println!("    {} {}{}", symbol, title_colored, issue_str);
        }
        println!();
    }

    if dry_run {
        println!("{}", "Run with --execute to apply.".dimmed());
    }
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max - 1).collect::<String>())
    }
}
