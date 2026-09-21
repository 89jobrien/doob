//! Marks one or more todos as completed.

use crate::ports::TodoRepository;
use anyhow::Result;

/// Completes each supplied todo ID and returns the number updated.
pub async fn execute(repo: &dyn TodoRepository, ids: Vec<String>) -> Result<usize> {
    let mut completed_count = 0;

    for id in ids {
        repo.complete_todo(&id).await?;
        completed_count += 1;
    }

    Ok(completed_count)
}
