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
                println!("TODO: Implement list command");
                println!("Status: {:?}", status);
                println!("Project: {:?}", project);
                println!("Limit: {:?}", limit);
                Ok(())
            }
            TodoAction::Complete { ids } => {
                println!("TODO: Implement complete command");
                println!("IDs: {:?}", ids);
                Ok(())
            }
        }
    }
}
