use crate::ports::TodoRepository;
use anyhow::Result;

pub async fn execute(repo: &dyn TodoRepository, ids: Vec<String>) -> Result<usize> {
    let mut completed_count = 0;

    for id in ids {
        repo.complete_todo(&id).await?;
        completed_count += 1;
    }

    Ok(completed_count)
}
