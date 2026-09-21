//! Removes one or more todos.

use crate::ports::TodoRepository;
use anyhow::Result;

/// Deletes each supplied todo ID and returns the number removed.
pub async fn execute(repo: &dyn TodoRepository, ids: Vec<String>) -> Result<usize> {
    let mut removed_count = 0;

    for id in ids {
        repo.delete_todo(&id).await?;
        removed_count += 1;
    }

    Ok(removed_count)
}
