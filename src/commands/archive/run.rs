use crate::models::Todo;
use crate::ports::ArchiveRepository;
use anyhow::Result;
use chrono::Utc;

pub struct ArchiveRunResult {
    pub dry_run: bool,
    pub candidates: Vec<Todo>,
    pub archived_count: usize,
}

pub async fn execute(
    repo: &dyn ArchiveRepository,
    older_than_days: u32,
    apply: bool,
    project: Option<String>,
) -> Result<ArchiveRunResult> {
    let cutoff = Utc::now() - chrono::Duration::days(older_than_days as i64);
    let cutoff_str = cutoff.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let candidates = repo
        .find_archive_candidates(&cutoff_str, project.as_deref())
        .await?;

    if !apply {
        return Ok(ArchiveRunResult {
            dry_run: true,
            candidates,
            archived_count: 0,
        });
    }

    let mut archived_count = 0usize;

    for todo in &candidates {
        repo.archive_todo(todo).await?;
        if todo.id.is_some() {
            archived_count += 1;
        }
    }

    Ok(ArchiveRunResult {
        dry_run: false,
        candidates,
        archived_count,
    })
}
