use crate::commands::handoff::sync::SyncSummary;
use crate::models::handoff_item::HandoffItem;
use serde_json::{json, Value};

pub fn format_list(items: &[HandoffItem]) -> Value {
    json!(items)
}

pub fn format_sync_summary(summary: &SyncSummary) -> Value {
    json!({
        "created": summary.created,
        "updated": summary.updated,
        "pulled": summary.pulled,
    })
}
