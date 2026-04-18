use crate::commands::note::normalize_note_id;
use crate::ports::TodoRepository;
use anyhow::Result;

pub async fn execute(repo: &dyn TodoRepository, ids: Vec<String>) -> Result<usize> {
    let mut removed_count = 0;

    for id in ids {
        let record_id = normalize_note_id(id);
        repo.delete_note(&record_id).await?;
        removed_count += 1;
    }

    Ok(removed_count)
}
