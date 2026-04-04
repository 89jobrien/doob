use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

    /// Manage notes
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },

    /// Visual kanban board of todos
    Kan {
        /// Filter by project
        #[arg(short = 'p', long)]
        project: Option<String>,

        /// Filter by status (comma-separated: pending,in_progress,completed,cancelled)
        #[arg(long, value_delimiter = ',')]
        status: Option<Vec<String>>,
    },

    /// Full-text search across todos and notes
    Search {
        /// Search query
        #[arg(required = true)]
        query: String,

        /// Filter by type: todo, note, or all
        #[arg(long = "type", default_value = "all")]
        search_type: String,

        /// Filter by project
        #[arg(short = 'p', long)]
        project: Option<String>,
    },

    /// Analytics and statistics
    Stats {
        /// Filter by project
        #[arg(short = 'p', long)]
        project: Option<String>,

        /// Time window in days for recent activity
        #[arg(long, default_value_t = 7)]
        window: u32,
    },

    /// Archive completed/cancelled todos
    Archive {
        #[command(subcommand)]
        action: ArchiveAction,
    },

    /// Manage handoff items (bidirectional sync with HANDOFF.yaml)
    Handoff {
        #[command(subcommand)]
        action: HandoffAction,
    },

    /// Launch the doobdash TUI dashboard
    Tui {
        /// Path to HANDOFF.yaml (auto-detected if omitted)
        #[arg(short = 'f', long)]
        file: Option<String>,
    },

    /// Live-updating kanban board
    Watch {
        /// Filter by project
        #[arg(short = 'p', long)]
        project: Option<String>,

        /// Filter by status (comma-separated)
        #[arg(long, value_delimiter = ',')]
        status: Option<Vec<String>>,

        /// Refresh interval in seconds
        #[arg(long, default_value_t = 5)]
        interval: u64,
    },

    /// Print machine-readable JSON manifest of all commands and params
    Schema,
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

        /// UUIDs this todo blocks (comma-separated)
        #[arg(long, value_delimiter = ',')]
        blocks: Option<Vec<String>>,

        /// UUIDs that block this todo (comma-separated)
        #[arg(long = "blocked-by", value_delimiter = ',')]
        blocked_by: Option<Vec<String>>,
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

    /// Show dependency chain for a todo
    Deps {
        /// Todo UUID or record ID
        #[arg(required = true)]
        id: String,
    },
}

#[derive(Subcommand)]
pub enum NoteAction {
    /// Add note(s)
    Add {
        /// Note content
        #[arg(required = true)]
        content: Vec<String>,

        #[arg(short = 'p', long)]
        project: Option<String>,

        #[arg(short = 'f', long)]
        file: Option<String>,

        #[arg(short = 't', long)]
        tags: Option<String>,
    },

    /// List notes
    List {
        #[arg(short = 'p', long)]
        project: Option<String>,

        #[arg(short = 'l', long)]
        limit: Option<usize>,
    },

    /// Remove/delete note(s)
    Remove {
        /// Note ID(s)
        #[arg(required = true)]
        ids: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum HandoffAction {
    /// Bidirectional sync between HANDOFF.yaml and the handoff_item table
    Sync {
        /// Path to the HANDOFF.yaml file
        #[arg(long)]
        file: PathBuf,
    },

    /// List handoff items
    List {
        /// Filter by project name
        #[arg(short = 'p', long)]
        project: Option<String>,

        /// Filter by status (open, done, parked, blocked)
        #[arg(long)]
        status: Option<String>,
    },

    /// Append an extra entry to a handoff item
    AddExtra {
        /// Handoff item ID (e.g. cci-7)
        handoff_id: String,

        /// Entry type: note, blocker, decision, discovery, escalation
        #[arg(long = "type")]
        entry_type: String,

        /// Note text
        #[arg(long)]
        note: String,
    },

    /// Update the status of a handoff item
    UpdateStatus {
        /// Handoff item ID (e.g. doob-1)
        handoff_id: String,

        /// New status: open, done, parked, blocked
        status: String,
    },
}

#[derive(Subcommand)]
pub enum ArchiveAction {
    /// Move old completed/cancelled todos to archive (dry-run by default)
    Run {
        /// Archive todos older than N days
        #[arg(long, default_value_t = 30)]
        older_than: u32,

        /// Actually perform the move (default is dry-run preview)
        #[arg(long)]
        apply: bool,

        /// Filter by project
        #[arg(short = 'p', long)]
        project: Option<String>,
    },

    /// List archived todos
    List {
        #[arg(short = 'p', long)]
        project: Option<String>,

        #[arg(short = 'l', long)]
        limit: Option<usize>,
    },
}
