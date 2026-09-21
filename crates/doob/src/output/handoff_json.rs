//! Serializes handoff items and sync summaries as JSON.

use crate::commands::handoff::sync::SyncSummary;
use crate::models::handoff_item::HandoffItem;
use serde_json::{json, Value};

/// Serializes handoff items as a JSON value.
pub fn format_list(items: &[HandoffItem]) -> Value {
    json!(items)
}

/// Serializes created, updated, and pulled item IDs.
pub fn format_sync_summary(summary: &SyncSummary) -> Value {
    json!({
        "created": summary.created,
        "updated": summary.updated,
        "pulled": summary.pulled,
    })
}
