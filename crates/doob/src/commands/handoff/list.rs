//! Lists persisted handoff items by project and status.

use crate::models::handoff_item::HandoffItem;
use crate::ports::HandoffRepository;
use anyhow::Result;

/// Returns handoff items matching the supplied filters.
pub async fn execute(
    repo: &dyn HandoffRepository,
    project: Option<String>,
    status: Option<String>,
) -> Result<Vec<HandoffItem>> {
    repo.list_handoff_items(project.as_deref(), status.as_deref())
        .await
}
