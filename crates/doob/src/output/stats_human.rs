use crate::commands::stats::StatsResult;

pub fn format_stats(stats: &StatsResult) -> String {
    let mut out = String::new();

    if let Some(ref proj) = stats.project {
        out.push_str(&format!("Project: {}\n\n", proj));
    }

    out.push_str(&format!("Total todos:     {:>5}\n", stats.total));
    out.push_str(&format!("  pending:       {:>5}\n", stats.pending));
    out.push_str(&format!("  in_progress:   {:>5}\n", stats.in_progress));
    out.push_str(&format!("  completed:     {:>5}\n", stats.completed));
    out.push_str(&format!("  cancelled:     {:>5}\n", stats.cancelled));

    out.push('\n');
    out.push_str(&format!(
        "Completion rate: {:>6.1}%\n",
        stats.completion_rate
    ));

    match stats.avg_completion_secs {
        Some(secs) => {
            out.push_str(&format!(
                "Avg time to complete: {}\n",
                format_duration(secs)
            ));
        }
        None => {
            out.push_str("Avg time to complete: —\n");
        }
    }

    out.push_str(&format!("Overdue:         {:>5}\n", stats.overdue_count));

    out.push('\n');
    out.push_str(&format!("Last {} days:\n", stats.window_days));
    out.push_str(&format!("  Created:       {:>5}\n", stats.created_window));
    out.push_str(&format!("  Completed:     {:>5}\n", stats.completed_window));

    out
}

fn format_duration(secs: f64) -> String {
    let total = secs as u64;
    let days = total / 86400;
    let hours = (total % 86400) / 3600;
    let mins = (total % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}
