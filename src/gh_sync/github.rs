use anyhow::{bail, Context, Result};
use std::process::Command;

/// Verify `gh` is on PATH. Call before any other function.
pub fn check_gh_available() -> Result<()> {
    let out = Command::new("gh")
        .arg("--version")
        .output()
        .context("gh CLI not found — install it from https://cli.github.com")?;
    if !out.status.success() {
        bail!(
            "gh --version failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

/// Create a GitHub issue. Returns the issue number.
pub fn create_issue(repo: &str, title: &str, body: &str) -> Result<u64> {
    let out = Command::new("gh")
        .args([
            "issue", "create", "--repo", repo, "--title", title, "--body", body,
        ])
        .output()
        .context("Failed to run gh issue create")?;
    if !out.status.success() {
        bail!(
            "gh issue create failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    // Output is the issue URL, e.g. https://github.com/owner/repo/issues/42
    let stdout = String::from_utf8_lossy(&out.stdout);
    parse_issue_number_from_url(stdout.trim()).with_context(|| {
        format!(
            "Could not parse issue number from gh output: {}",
            stdout.trim()
        )
    })
}

/// Close a GitHub issue by number.
pub fn close_issue(repo: &str, issue_number: u64) -> Result<()> {
    let num = issue_number.to_string();
    let out = Command::new("gh")
        .args(["issue", "close", "--repo", repo, &num])
        .output()
        .context("Failed to run gh issue close")?;
    if !out.status.success() {
        bail!(
            "gh issue close failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

/// Add a comment to a GitHub issue.
pub fn add_comment(repo: &str, issue_number: u64, body: &str) -> Result<()> {
    let num = issue_number.to_string();
    let out = Command::new("gh")
        .args(["issue", "comment", "--repo", repo, &num, "--body", body])
        .output()
        .context("Failed to run gh issue comment")?;
    if !out.status.success() {
        bail!(
            "gh issue comment failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn parse_issue_number_from_url(url: &str) -> Option<u64> {
    url.split('/').next_back()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_number_from_url() {
        assert_eq!(
            parse_issue_number_from_url("https://github.com/joe/repo/issues/42"),
            Some(42)
        );
    }

    #[test]
    fn parses_issue_number_from_url_no_trailing_whitespace() {
        // trim() happens in caller; raw newline should not parse
        assert_eq!(
            parse_issue_number_from_url("https://github.com/joe/repo/issues/7\n"),
            None
        );
    }
}
