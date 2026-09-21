//! Formats handoff items and sync summaries for terminal output.

use crate::commands::handoff::sync::SyncSummary;
use crate::models::handoff_item::HandoffItem;

// qual:allow(iosp) reason: "output formatter — string building with conditionals"
/// Formats handoff items as a human-readable list.
pub fn format_list(items: &[HandoffItem]) -> String {
    if items.is_empty() {
        return "No handoff items found.".to_string();
    }

    let mut out = String::new();
    for item in items {
        out.push_str(&format!(
            "[{}] {} ({})\n  project: {}  priority: {}  status: {}\n",
            item.handoff_id, item.title, item.uuid, item.project, item.priority, item.status
        ));
        if !item.extra.is_empty() {
            for e in &item.extra {
                let et = format!("{:?}", e.entry_type).to_lowercase();
                out.push_str(&format!("  + [{}] {}: {}\n", e.date, et, e.note));
            }
        }
    }
    out
}

/// Formats created, updated, and pulled item IDs as text.
pub fn format_sync_summary(summary: &SyncSummary) -> String {
    let mut out = String::from("Handoff sync complete\n");
    out.push_str(&format!(
        "  Created : {}\n",
        summary.created.join(", ").or_empty()
    ));
    out.push_str(&format!(
        "  Updated : {}\n",
        summary.updated.join(", ").or_empty()
    ));
    out.push_str(&format!(
        "  Pulled  : {}\n",
        summary.pulled.join(", ").or_empty()
    ));
    out
}

trait OrEmpty {
    fn or_empty(&self) -> &str;
}

impl OrEmpty for String {
    fn or_empty(&self) -> &str {
        if self.is_empty() {
            "(none)"
        } else {
            self.as_str()
        }
    }
}
