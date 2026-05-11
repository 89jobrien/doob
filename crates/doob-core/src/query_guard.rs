/// Input validation for values interpolated into SurrealDB query strings.
///
/// SurrealDB 2.x parameterized queries silently no-op (issue #6271), so raw
/// string interpolation is unavoidable. These guards enforce strict allowlists
/// before any user-supplied value reaches the query string.
use anyhow::{bail, Result};

/// Valid todo statuses (snake_case as stored in DB)
const ALLOWED_TODO_STATUSES: &[&str] = &["pending", "in_progress", "completed", "cancelled"];

/// Valid handoff/kanban statuses used by doobdash
const ALLOWED_HANDOFF_STATUSES: &[&str] = &["open", "done", "parked", "blocked"];

pub fn validate_status(s: &str) -> Result<()> {
    if ALLOWED_TODO_STATUSES.contains(&s) || ALLOWED_HANDOFF_STATUSES.contains(&s) {
        Ok(())
    } else {
        bail!(
            "invalid status value: {s:?} — must be one of: \
             pending, in_progress, completed, cancelled, open, done, parked, blocked"
        )
    }
}

pub fn validate_project(p: &str) -> Result<()> {
    if p.chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | ' ' | '/'))
    {
        Ok(())
    } else {
        bail!("invalid project value: {p:?} — only alphanumerics, hyphens, underscores, dots, and spaces are allowed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_accepts_todo_statuses() {
        for s in ALLOWED_TODO_STATUSES {
            assert!(validate_status(s).is_ok(), "expected {s:?} to be valid");
        }
    }

    #[test]
    fn status_accepts_handoff_statuses() {
        for s in ALLOWED_HANDOFF_STATUSES {
            assert!(validate_status(s).is_ok(), "expected {s:?} to be valid");
        }
    }

    #[test]
    fn status_rejects_injection() {
        assert!(validate_status("open' OR '1'='1").is_err());
        assert!(validate_status("'; DROP TABLE todo; --").is_err());
        assert!(validate_status("unknown").is_err());
    }

    #[test]
    fn project_accepts_normal_names() {
        for p in ["doob", "my-project", "joe_dev", "project.v2", "My Project"] {
            assert!(validate_project(p).is_ok());
        }
    }

    #[test]
    fn project_rejects_injection() {
        assert!(validate_project("doob' OR '1'='1").is_err());
        assert!(validate_project("foo'; DROP TABLE todo; --").is_err());
        assert!(validate_project("foo\nbar").is_err());
    }
}
