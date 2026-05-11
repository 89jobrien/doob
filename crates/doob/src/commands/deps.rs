use crate::models::Todo;
use crate::ports::TodoRepository;
use anyhow::Result;

/// Link deps to the first todo only from a batch-add result.
///
/// When multiple todos are created in one `add` invocation and `--blocks` /
/// `--blocked-by` are also specified, the intent is ambiguous. This function
/// enforces a clear rule: only the first todo gets the dep links. If the
/// caller passes an empty `todos` slice nothing is done.
pub async fn apply_batch_deps(
    repo: &dyn TodoRepository,
    todos: &[Todo],
    blocks: &[String],
    blocked_by: &[String],
) -> Result<()> {
    if blocks.is_empty() && blocked_by.is_empty() {
        return Ok(());
    }
    if let Some(first) = todos.first() {
        repo.link_deps(&first.uuid, blocks, blocked_by).await?;
    }
    Ok(())
}

pub struct DepsView {
    pub root: Todo,
    /// Todos that block this one (from blocked_by list)
    pub blockers: Vec<Todo>,
    /// Todos that this one blocks (from blocks list)
    pub dependents: Vec<Todo>,
}

pub async fn execute(repo: &dyn TodoRepository, id: String) -> Result<DepsView> {
    // Try UUID first, then record ID
    let root = match repo.get_todo_by_uuid(&id).await? {
        Some(t) => t,
        None => repo
            .get_todo(&id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Todo not found: {}", id))?,
    };

    let blockers = repo.get_todos_by_uuids(&root.blocked_by).await?;
    let dependents = repo.get_todos_by_uuids(&root.blocks).await?;

    Ok(DepsView {
        root,
        blockers,
        dependents,
    })
}
