mod error;

use doob::cache;
use error::ExitCode;
use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use doob::cli::{ArchiveAction, Cli, Commands, HandoffAction, NoteAction, TodoAction};
use doob::{commands, output};
use doob_surrealdb::{ArchiveRepositoryImpl, HandoffRepositoryImpl, TodoRepositoryImpl};

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => process::exit(ExitCode::Success as i32),
        Err(e) => {
            eprintln!("Error: {}", e);
            let code = ExitCode::from_error(&e);
            process::exit(code as i32);
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let db_conn = doob_surrealdb::create_connection(cli.db.as_deref()).await?;
    let repo = TodoRepositoryImpl::new(db_conn.clone());
    let repo: &dyn doob_core::ports::TodoRepository = &repo;
    let handoff_repo = HandoffRepositoryImpl::new(db_conn.clone());
    let handoff_repo: &dyn doob_core::ports::HandoffRepository = &handoff_repo;
    let archive_repo = ArchiveRepositoryImpl::new(db_conn.clone());
    let archive_repo: &dyn doob_core::ports::ArchiveRepository = &archive_repo;

    match cli.command {
        Commands::Todo { action } => match action {
            TodoAction::Add {
                content,
                priority,
                project,
                file,
                tags,
                blocks,
                blocked_by,
            } => {
                let todos =
                    commands::add::execute(repo, content, priority, project, file, tags).await?;

                let blocks_list = blocks.unwrap_or_default();
                let blocked_by_list = blocked_by.unwrap_or_default();
                commands::deps::apply_batch_deps(repo, &todos, &blocks_list, &blocked_by_list)
                    .await?;

                for todo in &todos {
                    println!("✓ Created todo: {}", todo.content);
                }

                cache::refresh(repo).await;
                Ok(())
            }
            TodoAction::List {
                status,
                project,
                limit,
            } => {
                let todos = commands::list::execute(repo, status, project, limit).await?;

                if cli.json {
                    println!("{}", output::format_json(&todos));
                } else {
                    println!("{}", output::format_human(&todos));
                }

                Ok(())
            }
            TodoAction::Complete { ids } => {
                let count = commands::complete::execute(repo, ids).await?;
                println!("✓ Completed {} todo(s)", count);
                cache::refresh(repo).await;
                Ok(())
            }
            TodoAction::Remove { ids } => {
                let count = commands::remove::execute(repo, ids).await?;
                println!("✓ Removed {} todo(s)", count);
                cache::refresh(repo).await;
                Ok(())
            }
            TodoAction::Due { id, date } => {
                commands::due::execute(repo, id.clone(), date.clone()).await?;
                if let Some(d) = date {
                    if d.to_lowercase() == "clear" {
                        println!("✓ Cleared due date for todo: {}", id);
                    } else {
                        println!("✓ Set due date for todo {}: {}", id, d);
                    }
                } else {
                    println!("✓ Cleared due date for todo: {}", id);
                }
                cache::refresh(repo).await;
                Ok(())
            }
            TodoAction::Undo { ids } => {
                let count = commands::undo::execute(repo, ids).await?;
                println!("✓ Undid completion for {} todo(s)", count);
                cache::refresh(repo).await;
                Ok(())
            }
            TodoAction::Update {
                id,
                priority,
                status,
                project,
                tags,
                content,
            } => {
                let fields = commands::update::UpdateFields {
                    priority,
                    status,
                    project,
                    tags,
                    content,
                };
                let todo = commands::update::execute(repo, id, fields).await?;
                println!("✓ Updated todo: {}", todo.content);
                cache::refresh(repo).await;
                Ok(())
            }
            TodoAction::Deps { id } => {
                let view = commands::deps::execute(repo, id).await?;
                if cli.json {
                    println!("{}", output::deps_json(&view));
                } else {
                    println!("{}", output::deps_human(&view));
                }
                Ok(())
            }
            TodoAction::GhSync {
                uuid,
                execute: do_execute,
                force,
                action,
            } => {
                commands::gh_sync::execute(
                    repo,
                    commands::gh_sync::GhSyncOptions {
                        uuid,
                        dry_run: !do_execute,
                        force,
                        action,
                        json: cli.json,
                    },
                )
                .await?;
                Ok(())
            }
        },

        Commands::Note { action } => match action {
            NoteAction::Add {
                content,
                project,
                file,
                tags,
            } => {
                let notes =
                    commands::note::add::execute(repo, content, project, file, tags).await?;

                for note in &notes {
                    println!("✓ Created note: {}", note.content);
                }

                Ok(())
            }
            NoteAction::List { project, limit } => {
                let notes = commands::note::list::execute(repo, project, limit).await?;

                if cli.json {
                    println!("{}", output::format_notes_json(&notes));
                } else {
                    println!("{}", output::format_notes_human(&notes));
                }

                Ok(())
            }
            NoteAction::Remove { ids } => {
                let count = commands::note::remove::execute(repo, ids).await?;
                println!("✓ Removed {} note(s)", count);
                Ok(())
            }
        },

        Commands::Kan { project, status } => {
            let status_filter: Option<Vec<doob::models::TodoStatus>> = status.map(|statuses| {
                statuses
                    .iter()
                    .filter_map(|s| commands::kan::parse_status(s))
                    .collect()
            });

            let (todos, filter) = commands::kan::execute(repo, project, status_filter).await?;

            let board = output::kanban::render_board(&todos, filter.as_deref());
            print!("{}", board);

            Ok(())
        }

        Commands::Search {
            query,
            search_type,
            project,
        } => {
            let results =
                commands::search::execute(repo, query.clone(), search_type, project).await?;
            if cli.json {
                println!("{}", output::search_json::format_results(&results, &query));
            } else {
                println!("{}", output::search_human::format_results(&results));
            }
            Ok(())
        }

        Commands::Stats { project, window } => {
            let stats = commands::stats::execute(repo, project, window).await?;
            if cli.json {
                println!("{}", output::stats_json::format_stats(&stats));
            } else {
                println!("{}", output::stats_human::format_stats(&stats));
            }
            Ok(())
        }

        Commands::Archive { action } => match action {
            ArchiveAction::Run {
                older_than,
                apply,
                project,
            } => {
                let result =
                    commands::archive::run::execute(archive_repo, older_than, apply, project)
                        .await?;
                if cli.json {
                    println!("{}", output::archive_json::format_run_result(&result));
                } else {
                    println!("{}", output::archive_human::format_run_result(&result));
                }
                Ok(())
            }
            ArchiveAction::List { project, limit } => {
                let archived =
                    commands::archive::list::execute(archive_repo, project, limit).await?;
                if cli.json {
                    println!("{}", output::archive_json::format_list(&archived));
                } else {
                    println!("{}", output::archive_human::format_list(&archived));
                }
                Ok(())
            }
        },

        Commands::Handoff { action } => match action {
            HandoffAction::Sync { file } => {
                let summary = commands::handoff::sync::execute(handoff_repo, &file).await?;
                if cli.json {
                    println!("{}", output::handoff_json::format_sync_summary(&summary));
                } else {
                    print!("{}", output::handoff_human::format_sync_summary(&summary));
                }
                Ok(())
            }
            HandoffAction::List { project, status } => {
                let items = commands::handoff::list::execute(handoff_repo, project, status).await?;
                if cli.json {
                    println!("{}", output::handoff_json::format_list(&items));
                } else {
                    print!("{}", output::handoff_human::format_list(&items));
                }
                Ok(())
            }
            HandoffAction::AddExtra {
                handoff_id,
                entry_type,
                note,
            } => {
                commands::handoff::add_extra::execute(
                    handoff_repo,
                    handoff_id.clone(),
                    entry_type,
                    note,
                )
                .await?;
                println!("✓ Added extra to {}", handoff_id);
                Ok(())
            }
            HandoffAction::UpdateStatus { handoff_id, status } => {
                commands::handoff::update_status::execute(
                    handoff_repo,
                    handoff_id.clone(),
                    status.clone(),
                )
                .await?;
                println!("✓ Updated {} status to {}", handoff_id, status);
                Ok(())
            }
        },

        Commands::Watch {
            project,
            status,
            interval,
        } => {
            let status_filter: Option<Vec<doob::models::TodoStatus>> = status.map(|statuses| {
                statuses
                    .iter()
                    .filter_map(|s| commands::kan::parse_status(s))
                    .collect()
            });
            commands::watch::execute(repo, project, status_filter, interval).await?;
            Ok(())
        }

        Commands::Tui { file } => {
            let mut cmd = std::process::Command::new("doobdash");
            if let Some(f) = file {
                cmd.arg(f);
            }
            let status = cmd
                .status()
                .context("Failed to launch doobdash — is it installed?")?;
            if !status.success() {
                anyhow::bail!("doobdash exited with: {}", status);
            }
            Ok(())
        }

        Commands::Schema => {
            let manifest = doob::commands::schema::build_manifest();
            println!("{}", serde_json::to_string_pretty(&manifest)?);
            Ok(())
        }
    }
}
