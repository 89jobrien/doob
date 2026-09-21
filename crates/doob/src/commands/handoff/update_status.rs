//! Updates the persisted status of a handoff item.

use crate::ports::HandoffRepository;
use anyhow::Result;

/// Sets the status for the identified handoff item.
pub async fn execute(
    repo: &dyn HandoffRepository,
    handoff_id: String,
    status: String,
) -> Result<()> {
    repo.update_handoff_status(&handoff_id, &status).await
}
