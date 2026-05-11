use crate::commands::stats::StatsResult;
use serde_json::json;

pub fn format_stats(stats: &StatsResult) -> String {
    let output = json!({
        "project": stats.project,
        "window_days": stats.window_days,
        "total": stats.total,
        "by_status": {
            "pending": stats.pending,
            "in_progress": stats.in_progress,
            "completed": stats.completed,
            "cancelled": stats.cancelled,
        },
        "completion_rate": stats.completion_rate,
        "avg_completion_secs": stats.avg_completion_secs,
        "overdue_count": stats.overdue_count,
        "window": {
            "created": stats.created_window,
            "completed": stats.completed_window,
        }
    });
    serde_json::to_string_pretty(&output).unwrap()
}
