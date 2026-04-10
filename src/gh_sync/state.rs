use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IssueRef {
    pub repo: String,
    pub issue_number: u64,
}

pub type StateMap = HashMap<String, IssueRef>;

pub fn state_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config/doob/gh-sync-state.json")
}

/// Load state from disk. Returns empty map if file does not exist.
pub fn load() -> Result<StateMap> {
    let path = state_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read state file {}", path.display()))?;
    let map: StateMap = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid JSON in state file {}", path.display()))?;
    Ok(map)
}

/// Save state to disk atomically (write to temp then rename).
pub fn save(map: &StateMap) -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create dir {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(map)?;
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, &json)
        .with_context(|| format!("Failed to write temp state file {}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, &path)
        .with_context(|| "Failed to rename state file".to_string())?;
    Ok(())
}

/// Returns true if the uuid already has an issue synced.
pub fn has_issue(map: &StateMap, uuid: &str) -> bool {
    map.contains_key(uuid)
}

/// Insert or update an entry.
pub fn upsert(map: &mut StateMap, uuid: &str, repo: &str, issue_number: u64) {
    map.insert(
        uuid.to_string(),
        IssueRef {
            repo: repo.to_string(),
            issue_number,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip_via_json(map: &StateMap) -> StateMap {
        let json = serde_json::to_string(map).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn upsert_and_has_issue() {
        let mut map = StateMap::new();
        assert!(!has_issue(&map, "uuid-1"));
        upsert(&mut map, "uuid-1", "joe/minibox", 7);
        assert!(has_issue(&map, "uuid-1"));
    }

    #[test]
    fn roundtrip_serialization() {
        let mut map = StateMap::new();
        upsert(&mut map, "uuid-abc", "joe/doob", 99);
        let rt = roundtrip_via_json(&map);
        assert_eq!(rt.get("uuid-abc").unwrap().issue_number, 99);
        assert_eq!(rt.get("uuid-abc").unwrap().repo, "joe/doob");
    }
}
