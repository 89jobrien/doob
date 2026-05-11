// Re-export domain types from doob-core at original paths
pub use doob_core::cache;
pub use doob_core::context;
pub use doob_core::error;
pub use doob_core::models;
pub use doob_core::ports;
pub use doob_core::query_guard;

// Re-export adapter crates at original paths
pub use doob_gh as gh_sync;
pub use doob_surrealdb as db;

// Re-export adapters from doob-surrealdb
pub mod adapters {
    pub use doob_surrealdb::{ArchiveRepositoryImpl, HandoffRepositoryImpl, TodoRepositoryImpl};
}

// Re-export sync sub-crates at original paths
pub mod sync {
    pub use doob_sync as domain;

    pub mod adapters {
        pub use doob_beads::BeadsAdapter;
    }
}

// Local modules (CLI, commands, output stay in doob)
pub mod cli;
pub mod commands;
pub mod output;
