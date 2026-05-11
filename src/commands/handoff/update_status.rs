use crate::ports::HandoffRepository;
use anyhow::Result;

pub async fn execute(
    repo: &dyn HandoffRepository,
    handoff_id: String,
    status: String,
) -> Result<()> {
    repo.update_handoff_status(&handoff_id, &status).await
}
