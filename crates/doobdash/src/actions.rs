use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Shell out: `doob handoff update-status <id> <status>`
pub fn set_status(_handoff_path: &Path, id: &str, status: &str) -> Result<()> {
    let out = Command::new("doob")
        .args(["handoff", "update-status", id, status])
        .output()
        .with_context(|| "Failed to run doob handoff update-status")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("doob update-status failed: {}", stderr);
    }
    Ok(())
}

/// Shell out: `doob handoff add-extra <id> --type note --note <text>`
pub fn add_note(_handoff_path: &Path, id: &str, text: &str) -> Result<()> {
    let out = Command::new("doob")
        .args(["handoff", "add-extra", id, "--type", "note", "--note", text])
        .output()
        .with_context(|| "Failed to run doob handoff add-extra")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("doob add-extra failed: {}", stderr);
    }
    Ok(())
}
