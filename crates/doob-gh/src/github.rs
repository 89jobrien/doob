use anyhow::{bail, Context, Result};
use std::process::{Command, Output};

fn run_gh(args: &[&str], label: &str) -> Result<Output> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run gh {label}"))?;
    if !out.status.success() {
        bail!(
            "gh {label} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(out)
}

/// Verify `gh` is on PATH. Call before any other function.
pub fn check_gh_available() -> Result<()> {
    run_gh(&["--version"], "--version")?;
    Ok(())
}

/// Create a GitHub issue. Returns the issue number.
pub fn create_issue(repo: &str, title: &str, body: &str) -> Result<u64> {
    let out = run_gh(
        &[
            "issue", "create", "--repo", repo, "--title", title, "--body", body,
        ],
        "issue create",
    )?;
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
    run_gh(&["issue", "close", "--repo", repo, &num], "issue close")?;
    Ok(())
}

/// Add a comment to a GitHub issue.
pub fn add_comment(repo: &str, issue_number: u64, body: &str) -> Result<()> {
    let num = issue_number.to_string();
    run_gh(
        &["issue", "comment", "--repo", repo, &num, "--body", body],
        "issue comment",
    )?;
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
        assert_eq!(
            parse_issue_number_from_url("https://github.com/joe/repo/issues/7\n"),
            None
        );
    }
}
