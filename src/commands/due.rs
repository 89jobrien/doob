use crate::ports::TodoRepository;
use anyhow::Result;

pub async fn execute(
    repo: &dyn TodoRepository,
    id: String,
    due_date: Option<String>,
) -> Result<()> {
    let date_arg = match &due_date {
        Some(d) if d.to_lowercase() == "clear" => None,
        Some(d) => Some(d.as_str()),
        None => None,
    };
    repo.set_due_date(&id, date_arg).await
}
