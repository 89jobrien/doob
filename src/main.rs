mod error;

use error::ExitCode;
use std::process;

use anyhow::Result;
use clap::Parser;
use doob::cli::{Cli, Commands, TodoAction};
use doob::{commands, db, output};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        let code = ExitCode::from_error(&e);
        process::exit(code as i32);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let db = db::create_connection(cli.db.as_deref()).await?;

    match cli.command {
        Commands::Todo { action } => match action {
            TodoAction::Add { content, priority, project, file, tags } => {
                let todos = commands::add::execute(&db, content, priority, project, file, tags).await?;

                for todo in &todos {
                    println!("✓ Created todo: {}", todo.content);
                }

                Ok(())
            }
            TodoAction::List { status, project, limit } => {
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
        },
    }
}
