use crate::sync::domain::{
    HealthCheck, IssueCreator, Provider, SyncError, SyncRecord, SyncableTodo,
};
use std::process::Command;

const PROVIDER_NAME: &str = "beads";
const BEADS_ISSUE_TYPE: &str = "task";

#[derive(Default)]
pub struct BeadsAdapter {}

impl BeadsAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    fn map_priority(&self, priority: u8) -> u8 {
        priority.min(4)
    }

    fn extract_issue_id(&self, output: &str) -> Result<String, SyncError> {
        output
            .split_whitespace()
            .find(|s| s.starts_with("bd-") || s.starts_with("beads-"))
            .map(String::from)
            .ok_or_else(|| {
                SyncError::ExternalApiError("Could not parse bd issue ID from output".to_string())
            })
    }
}

impl Provider for BeadsAdapter {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
}

impl HealthCheck for BeadsAdapter {
    fn is_available(&self) -> Result<bool, SyncError> {
        Command::new("bd")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .map_err(|e| SyncError::ProviderUnavailable(format!("bd CLI not found: {}", e)))
    }
}

impl IssueCreator for BeadsAdapter {
    fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError> {
        let mut cmd = Command::new("bd");
        cmd.arg("create")
            .arg(&todo.title)
            .arg(format!("--type={}", BEADS_ISSUE_TYPE))
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

        let output = cmd
            .output()
            .map_err(|e| SyncError::ExternalApiError(format!("Failed to run bd: {}", e)))?;

        if !output.status.success() {
            return Err(SyncError::ExternalApiError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let bd_id = self.extract_issue_id(&stdout)?;

        Ok(SyncRecord {
            external_id: bd_id,
            external_url: None,
            provider: self.name().to_string(),
            synced_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn extract_issue_id__parses_bd_prefix() {
        let adapter = BeadsAdapter::new();
        let output = "Created issue bd-42 successfully";
        let result = adapter.extract_issue_id(output);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bd-42");
    }

    #[test]
    fn extract_issue_id__parses_beads_prefix() {
        let adapter = BeadsAdapter::new();
        let output = "Created issue beads-99 in project";
        let result = adapter.extract_issue_id(output);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "beads-99");
    }

    #[test]
    fn extract_issue_id__returns_err_on_unrecognized_output() {
        let adapter = BeadsAdapter::new();
        let output = "success";
        let result = adapter.extract_issue_id(output);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SyncError::ExternalApiError(_)
        ));
    }

    #[test]
    fn map_priority__clamps_at_4() {
        let adapter = BeadsAdapter::new();
        assert_eq!(adapter.map_priority(10), 4);
        assert_eq!(adapter.map_priority(5), 4);
        assert_eq!(adapter.map_priority(4), 4);
        assert_eq!(adapter.map_priority(2), 2);
        assert_eq!(adapter.map_priority(0), 0);
    }
}
