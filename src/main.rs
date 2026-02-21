use anyhow::Result;
use clap::Parser;
use doob::cli::{Cli, Commands, TodoAction};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let db = doob::db::create_connection(cli.db.as_deref()).await?;

    match cli.command {
        Commands::Todo { action } => match action {
            TodoAction::Add { content, priority, project, file, tags } => {
                let todos = doob::commands::add::execute(
                    &db,
                    content,
                    priority,
                    project,
                    file,
                    tags,
                ).await?;

                for todo in &todos {
                    println!("✓ Created todo: {}", todo.content);
                }

                Ok(())
            }
            TodoAction::List { status, project, limit } => {
                let todos = doob::commands::list::execute(&db, status, project, limit).await?;

                if cli.json {
                    println!("{}", doob::output::format_json(&todos));
                } else {
                    println!("{}", doob::output::format_human(&todos));
                }

                Ok(())
            }
            TodoAction::Complete { ids } => {
                let count = doob::commands::complete::execute(&db, ids).await?;
                println!("✓ Completed {} todo(s)", count);
                Ok(())
            }
        }
    }
}
