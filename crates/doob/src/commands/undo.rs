//! Restores completed todos to the pending state.

use crate::ports::TodoRepository;
use anyhow::Result;

/// Reopens each supplied todo ID and returns the number updated.
pub async fn execute(repo: &dyn TodoRepository, ids: Vec<String>) -> Result<usize> {
    let mut undone_count = 0;

    for id in ids {
        repo.undo_todo(&id).await?;
        undone_count += 1;
    }

    Ok(undone_count)
}
