use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "doob")]
#[command(about = "Modern todo management for coding agents")]
pub struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    /// Database path
    #[arg(long, global = true)]
    pub db: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage todos
    Todo {
        #[command(subcommand)]
        action: TodoAction,
    },
}

#[derive(Subcommand)]
pub enum TodoAction {
    /// Add todo(s)
    Add {
        /// Task description(s)
        #[arg(required = true)]
        content: Vec<String>,

        #[arg(long)]
        priority: Option<u8>,

        #[arg(short = 'p', long)]
        project: Option<String>,

        #[arg(short = 'f', long)]
        file: Option<String>,

        #[arg(short = 't', long)]
        tags: Option<String>,
    },

    /// List todos
    List {
        #[arg(long)]
        status: Option<String>,

        #[arg(short = 'p', long)]
        project: Option<String>,

        #[arg(short = 'l', long)]
        limit: Option<usize>,
    },

    /// Complete todo(s)
    Complete {
        /// Todo ID(s)
        #[arg(required = true)]
        ids: Vec<String>,
    },

    /// Remove/delete todo(s)
    Remove {
        /// Todo ID(s)
        #[arg(required = true)]
        ids: Vec<String>,
    },

    /// Set or clear due date for a todo
    Due {
        /// Todo ID
        #[arg(required = true)]
        id: String,

        /// Due date (YYYY-MM-DD or 'clear')
        #[arg(required = false)]
        date: Option<String>,
    },

    /// Undo completion (mark as pending)
    Undo {
        /// Todo ID(s)
        #[arg(required = true)]
        ids: Vec<String>,
    },
}
