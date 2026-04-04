mod error;

use error::ExitCode;
use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use doob::cli::{ArchiveAction, Cli, Commands, HandoffAction, NoteAction, TodoAction};
use doob::{commands, db, output};

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

    let db = db::create_connection(cli.db.as_deref()).await?;

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
                    commands::add::execute(&db, content, priority, project, file, tags).await?;

                // Link deps if provided
                let blocks_list = blocks.unwrap_or_default();
                let blocked_by_list = blocked_by.unwrap_or_default();
                if !blocks_list.is_empty() || !blocked_by_list.is_empty() {
                    for todo in &todos {
                        commands::deps::link(&db, &todo.uuid, &blocks_list, &blocked_by_list)
                            .await?;
                    }
                }

                for todo in &todos {
                    println!("✓ Created todo: {}", todo.content);
                }

                Ok(())
            }
            TodoAction::List {
                status,
                project,
                limit,
            } => {
                let todos = commands::list::execute(&db, status, project, limit).await?;

                if cli.json {
                    println!("{}", output::format_json(&todos));
                } else {
                    println!("{}", output::format_human(&todos));
                }

                Ok(())
            }
            TodoAction::Complete { ids } => {
                let count = commands::complete::execute(&db, ids).await?;
                println!("✓ Completed {} todo(s)", count);
                Ok(())
            }
            TodoAction::Remove { ids } => {
                let count = commands::remove::execute(&db, ids).await?;
                println!("✓ Removed {} todo(s)", count);
                Ok(())
            }
            TodoAction::Due { id, date } => {
                commands::due::execute(&db, id.clone(), date.clone()).await?;
                if let Some(d) = date {
                    if d.to_lowercase() == "clear" {
                        println!("✓ Cleared due date for todo: {}", id);
                    } else {
                        println!("✓ Set due date for todo {}: {}", id, d);
                    }
                } else {
                    println!("✓ Cleared due date for todo: {}", id);
                }
                Ok(())
            }
            TodoAction::Undo { ids } => {
                let count = commands::undo::execute(&db, ids).await?;
                println!("✓ Undid completion for {} todo(s)", count);
                Ok(())
            }
            TodoAction::Deps { id } => {
                let view = commands::deps::execute(&db, id).await?;
                if cli.json {
                    println!("{}", output::deps_json(&view));
                } else {
                    println!("{}", output::deps_human(&view));
                }
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
                let notes = commands::note::add::execute(&db, content, project, file, tags).await?;

                for note in &notes {
                    println!("✓ Created note: {}", note.content);
                }

                Ok(())
            }
            NoteAction::List { project, limit } => {
                let notes = commands::note::list::execute(&db, project, limit).await?;

                if cli.json {
                    println!("{}", output::format_notes_json(&notes));
                } else {
                    println!("{}", output::format_notes_human(&notes));
                }

                Ok(())
            }
            NoteAction::Remove { ids } => {
                let count = commands::note::remove::execute(&db, ids).await?;
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

            let (todos, filter) = commands::kan::execute(&db, project, status_filter).await?;

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
                commands::search::execute(&db, query.clone(), search_type, project).await?;
            if cli.json {
                println!("{}", output::search_json::format_results(&results, &query));
            } else {
                println!("{}", output::search_human::format_results(&results));
            }
            Ok(())
        }

        Commands::Stats { project, window } => {
            let stats = commands::stats::execute(&db, project, window).await?;
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
                    commands::archive::run::execute(&db, older_than, apply, project).await?;
                if cli.json {
                    println!("{}", output::archive_json::format_run_result(&result));
                } else {
                    println!("{}", output::archive_human::format_run_result(&result));
                }
                Ok(())
            }
            ArchiveAction::List { project, limit } => {
                let archived = commands::archive::list::execute(&db, project, limit).await?;
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
                let summary = commands::handoff::sync::execute(&db, &file).await?;
                if cli.json {
                    println!(
                        "{}",
                        output::handoff_json::format_sync_summary(&summary)
                    );
                } else {
                    print!("{}", output::handoff_human::format_sync_summary(&summary));
                }
                Ok(())
            }
            HandoffAction::List { project, status } => {
                let items = commands::handoff::list::execute(&db, project, status).await?;
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
                commands::handoff::add_extra::execute(&db, handoff_id.clone(), entry_type, note)
                    .await?;
                println!("✓ Added extra to {}", handoff_id);
                Ok(())
            }
            HandoffAction::UpdateStatus { handoff_id, status } => {
                commands::handoff::update_status::execute(&db, handoff_id.clone(), status.clone())
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
            commands::watch::execute(&db, project, status_filter, interval).await?;
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
    }
}
