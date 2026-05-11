use crate::ports::TodoRepository;
use anyhow::Result;

pub async fn execute(repo: &dyn TodoRepository, ids: Vec<String>) -> Result<usize> {
    let mut removed_count = 0;

    for id in ids {
        repo.delete_todo(&id).await?;
        removed_count += 1;
    }

    Ok(removed_count)
}
