use crate::commands::kan;
use crate::db::DbConnection;
use crate::models::TodoStatus;
use crate::output::kanban;
use anyhow::Result;
use std::io::Write;
use tokio::signal;
use tokio::time::{interval, Duration};

pub async fn execute(
    db: &DbConnection,
    project: Option<String>,
    status_filter: Option<Vec<TodoStatus>>,
    interval_secs: u64,
) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(interval_secs));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let (todos, filter) = kan::execute(db, project.clone(), status_filter.clone()).await?;
                let board = kanban::render_board(&todos, filter.as_deref());
                print!("\x1b[2J\x1b[H");
                print!("{}", board);
                std::io::stdout().flush()?;
            }
            _ = signal::ctrl_c() => {
                println!("\nExiting watch mode.");
                break;
            }
        }
    }

    Ok(())
}
