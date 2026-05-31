use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

// qual:allow(iosp) reason: "shell adapter — command execution with error check"
fn run_doob(args: &[&str], label: &str) -> Result<()> {
    let out = Command::new("doob")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run doob {label}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("doob {label} failed: {stderr}");
    }
    Ok(())
}

/// Shell out: `doob handoff update-status <id> <status>`
pub fn set_status(_handoff_path: &Path, id: &str, status: &str) -> Result<()> {
    run_doob(&["handoff", "update-status", id, status], "update-status")
}

/// Shell out: `doob handoff add-extra <id> --type note --note <text>`
pub fn add_note(_handoff_path: &Path, id: &str, text: &str) -> Result<()> {
    run_doob(
        &["handoff", "add-extra", id, "--type", "note", "--note", text],
        "add-extra",
    )
}
