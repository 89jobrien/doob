mod cli;
mod commands;
mod db;
mod models;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, TodoAction};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let db = db::create_connection(cli.db.as_deref()).await?;

    match cli.command {
        Commands::Todo { action } => match action {
            TodoAction::Add { .. } => {
                println!("Add command - not implemented");
                Ok(())
            }
            TodoAction::List { .. } => {
                println!("List command - not implemented");
                Ok(())
            }
            TodoAction::Complete { .. } => {
                println!("Complete command - not implemented");
                Ok(())
            }
        },
    }
}
