// src/sync/adapters/beads.rs

use crate::sync::domain::{IssueTracker, SyncableTodo, SyncRecord, SyncError};
use std::process::Command;

pub struct BeadsAdapter {
    // No state needed - delegates to bd CLI
}

impl BeadsAdapter {
    pub fn new() -> Self {
        Self {}
    }

    fn map_priority(&self, priority: u8) -> u8 {
        // Direct 0-4 mapping
        priority.min(4)
    }

    fn extract_issue_id(&self, output: &str) -> Result<String, SyncError> {
        // Parse "Created issue bd-42" or similar
        output.split_whitespace()
            .find(|s| s.starts_with("bd-") || s.starts_with("beads-"))
            .map(String::from)
            .ok_or_else(|| SyncError::ExternalApiError(
                "Could not parse bd issue ID from output".to_string()
            ))
    }
}

impl IssueTracker for BeadsAdapter {
    fn name(&self) -> &str {
        "beads"
    }

    fn is_available(&self) -> Result<bool, SyncError> {
        Command::new("bd")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .map_err(|e| SyncError::ProviderUnavailable(format!("bd CLI not found: {}", e)))
    }

    fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
        let mut cmd = Command::new("bd");
        cmd.arg("create")
           .arg(&todo.title)
           .arg("--type=task")
           .arg(format!("--priority={}", self.map_priority(todo.priority)));

        if let Some(ref desc) = todo.description {
            cmd.arg(format!("--description={}", desc));
        }

        if let Some(ref _project) = todo.project {
            cmd.arg(format!("--external-ref=doob-{}", todo.id));
        }

        if !todo.tags.is_empty() {
            cmd.arg(format!("--notes=tags: {}", todo.tags.join(", ")));
        }

        let output = cmd.output()
            .map_err(|e| SyncError::ExternalApiError(format!("Failed to run bd: {}", e)))?;

        if !output.status.success() {
            return Err(SyncError::ExternalApiError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let bd_id = self.extract_issue_id(&stdout)?;

        Ok(SyncRecord {
            external_id: bd_id,
            external_url: None,
            provider: "beads".to_string(),
            synced_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
