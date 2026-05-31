use crate::commands::deps::DepsView;
use crate::models::TodoStatus;

pub fn format_deps(view: &DepsView) -> String {
    let mut out = String::new();

    let status = status_str(&view.root.status);
    out.push_str(&format!("Todo: [{}] {}\n", status, view.root.content));
    out.push_str(&format!("  uuid: {}\n", view.root.uuid));

    out.push('\n');

    if view.blockers.is_empty() {
        out.push_str("Blocked by: (none)\n");
    } else {
        out.push_str("Blocked by:\n");
        for t in &view.blockers {
            let done = matches!(t.status, TodoStatus::Completed | TodoStatus::Cancelled);
            let marker = if done { "✓" } else { "✗" };
            out.push_str(&format!(
                "  {} [{}] {} ({})\n",
                marker,
                status_str(&t.status),
                t.content,
                &t.uuid[..8]
            ));
        }
    }

    out.push('\n');

    if view.dependents.is_empty() {
        out.push_str("Blocks: (none)\n");
    } else {
        out.push_str("Blocks:\n");
        for t in &view.dependents {
            out.push_str(&format!(
                "  → [{}] {} ({})\n",
                status_str(&t.status),
                t.content,
                &t.uuid[..8]
            ));
        }
    }

    out
}

fn status_str(status: &TodoStatus) -> &str {
    status.as_str()
}
