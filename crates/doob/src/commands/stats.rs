use crate::models::TodoStatus;
use crate::ports::TodoRepository;
use anyhow::Result;
use chrono::Utc;

const PERCENT: f64 = 100.0;
const ZERO_RATE: f64 = 0.0;

pub struct StatsResult {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub cancelled: usize,
    pub completion_rate: f64,
    pub avg_completion_secs: Option<f64>,
    pub overdue_count: usize,
    pub created_window: usize,
    pub completed_window: usize,
    pub window_days: u32,
    pub project: Option<String>,
}

pub async fn execute(
    repo: &dyn TodoRepository,
    project: Option<String>,
    window_days: u32,
) -> Result<StatsResult> {
    // Get all todos - we'll filter by project in application code
    let todos = repo.list_todos(None, project.as_deref(), None).await?;

    let now = Utc::now();
    let window_start = now - chrono::Duration::days(window_days as i64);

    let mut pending = 0usize;
    let mut in_progress = 0usize;
    let mut completed = 0usize;
    let mut cancelled = 0usize;
    let mut overdue_count = 0usize;
    let mut created_window = 0usize;
    let mut completed_window = 0usize;
    let mut completion_durations: Vec<f64> = Vec::new();

    for todo in &todos {
        match todo.status {
            TodoStatus::Pending => pending += 1,
            TodoStatus::InProgress => in_progress += 1,
            TodoStatus::Completed => completed += 1,
            TodoStatus::Cancelled => cancelled += 1,
        }

        if matches!(todo.status, TodoStatus::Pending | TodoStatus::InProgress) {
            if let Some(due) = todo.due_date {
                if due < now {
                    overdue_count += 1;
                }
            }
        }

        if todo.created_at >= window_start {
            created_window += 1;
        }

        if let Some(completed_at) = todo.completed_at {
            if completed_at >= window_start {
                completed_window += 1;
            }
            let duration_secs = (completed_at - todo.created_at).num_seconds() as f64;
            if duration_secs >= 0.0 {
                completion_durations.push(duration_secs);
            }
        }
    }

    let total = todos.len();
    let completion_rate = if total > 0 {
        completed as f64 / total as f64 * PERCENT
    } else {
        ZERO_RATE
    };

    let avg_completion_secs = if completion_durations.is_empty() {
        None
    } else {
        Some(completion_durations.iter().sum::<f64>() / completion_durations.len() as f64)
    };

    Ok(StatsResult {
        total,
        pending,
        in_progress,
        completed,
        cancelled,
        completion_rate,
        avg_completion_secs,
        overdue_count,
        created_window,
        completed_window,
        window_days,
        project,
    })
}
