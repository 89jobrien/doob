use crate::config::GhSyncConfig;

/// Returns `Some("owner/repo")` if the project maps to an allowlisted GitHub repo.
/// Returns `None` if the project is not in the allowlist (sync should be skipped).
pub fn resolve(project: &str, cfg: &GhSyncConfig) -> Option<String> {
    let repo_name = strip_prefix(project);
    let allowed = is_allowed(repo_name, cfg);
    if allowed {
        Some(format!("{}/{}", cfg.github.owner, repo_name))
    } else {
        None
    }
}

/// Strip `dev/` prefix if present.
fn strip_prefix(project: &str) -> &str {
    project.strip_prefix("dev/").unwrap_or(project)
}

/// Check if repo_name is in the allowlist.
/// If allowlist is None, all repos are considered allowed.
fn is_allowed(repo_name: &str, cfg: &GhSyncConfig) -> bool {
    match &cfg.github.allowlist {
        None => true,
        Some(list) => list.iter().any(|r| r == repo_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GhSyncConfig, GithubConfig, SyncConfig};

    fn cfg(owner: &str, allowlist: Option<Vec<&str>>) -> GhSyncConfig {
        GhSyncConfig {
            github: GithubConfig {
                owner: owner.to_string(),
                allowlist: allowlist.map(|v| v.into_iter().map(String::from).collect()),
            },
            sync: SyncConfig::default(),
        }
    }

    #[test]
    fn strips_dev_prefix() {
        let c = cfg("joe", Some(vec!["minibox"]));
        assert_eq!(resolve("dev/minibox", &c), Some("joe/minibox".into()));
    }

    #[test]
    fn bare_project_name_works() {
        let c = cfg("joe", Some(vec!["coursers"]));
        assert_eq!(resolve("coursers", &c), Some("joe/coursers".into()));
    }

    #[test]
    fn project_not_in_allowlist_returns_none() {
        let c = cfg("joe", Some(vec!["minibox"]));
        assert_eq!(resolve("dev/maestro", &c), None);
    }

    #[test]
    fn no_allowlist_allows_everything() {
        let c = cfg("joe", None);
        assert_eq!(resolve("dev/anything", &c), Some("joe/anything".into()));
    }
}
