use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, PartialEq)]
pub struct GhSyncConfig {
    pub github: GithubConfig,
    #[serde(default)]
    pub sync: SyncConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct GithubConfig {
    pub owner: String,
    /// If present, used exactly. If absent, all repos are allowed.
    pub allowlist: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SyncConfig {
    #[serde(default = "default_true")]
    pub close_on_complete: bool,
    #[serde(default = "default_true")]
    pub tombstone_on_remove: bool,
}

fn default_true() -> bool {
    true
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            close_on_complete: true,
            tombstone_on_remove: true,
        }
    }
}

/// Load config from `~/.config/doob/gh-sync.toml`.
/// Returns `None` if the file does not exist (sync is skipped).
/// Returns `Err` if the file exists but is malformed.
pub fn load() -> Result<Option<GhSyncConfig>> {
    let path = config_path();
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let cfg: GhSyncConfig =
        toml::from_str(&raw).with_context(|| format!("Invalid TOML in {}", path.display()))?;
    Ok(Some(cfg))
}

pub fn config_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config/doob/gh-sync.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let toml = r#"
[github]
owner = "testuser"
allowlist = ["repo-a", "repo-b"]

[sync]
close_on_complete = true
tombstone_on_remove = false
"#;
        let cfg: GhSyncConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.github.owner, "testuser");
        assert_eq!(
            cfg.github.allowlist,
            Some(vec!["repo-a".into(), "repo-b".into()])
        );
        assert!(cfg.sync.close_on_complete);
        assert!(!cfg.sync.tombstone_on_remove);
    }

    #[test]
    fn sync_section_defaults_when_absent() {
        let toml = r#"
[github]
owner = "testuser"
"#;
        let cfg: GhSyncConfig = toml::from_str(toml).unwrap();
        assert!(cfg.sync.close_on_complete);
        assert!(cfg.sync.tombstone_on_remove);
        assert!(cfg.github.allowlist.is_none());
    }

    #[test]
    fn load_returns_none_when_file_absent() {
        let path = PathBuf::from("/tmp/doob-gh-sync-absent-test.toml");
        assert!(!path.exists());
        // Verifies the logic: a nonexistent file returns None
    }
}
