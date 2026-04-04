use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct YamlItem {
    pub id: String,
    #[serde(default)]
    pub priority: String,
    #[serde(default)]
    pub status: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LogEntry {
    pub date: String,
    pub summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct HandoffData {
    pub items: Vec<YamlItem>,
    pub log: Vec<LogEntry>,
    pub state: StateData,
    pub handoff_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct StateData {
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub build: String,
    #[serde(default)]
    pub tests: String,
    #[serde(default)]
    pub notes: String,
}

pub fn load(handoff_path: &Path) -> Result<HandoffData> {
    let raw = std::fs::read_to_string(handoff_path)?;
    let val: serde_yaml::Value = serde_yaml::from_str(&raw)?;

    let items: Vec<YamlItem> = if val.is_sequence() {
        serde_yaml::from_value(val.clone())?
    } else if let Some(items_val) = val.get("items") {
        serde_yaml::from_value(items_val.clone())?
    } else {
        vec![]
    };

    let log: Vec<LogEntry> = val
        .get("log")
        .and_then(|l| serde_yaml::from_value(l.clone()).ok())
        .unwrap_or_default();

    // State file: <repo_root>/.ctx/HANDOFF.state.yaml
    let repo_root = handoff_path.parent().unwrap_or(Path::new("."));
    let state_path = repo_root.join(".ctx/HANDOFF.state.yaml");
    let state: StateData = if state_path.exists() {
        let s = std::fs::read_to_string(&state_path)?;
        serde_yaml::from_str(&s).unwrap_or_default()
    } else {
        StateData::default()
    };

    Ok(HandoffData {
        items,
        log,
        state,
        handoff_path: handoff_path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_items_from_map() {
        let yaml = r#"
project: doob
items:
- id: doob-1
  priority: P1
  status: open
  title: Test item
log:
- date: "2026-04-01"
  summary: Did a thing
"#;
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let data = load(f.path()).unwrap();
        assert_eq!(data.items.len(), 1);
        assert_eq!(data.items[0].id, "doob-1");
        assert_eq!(data.log.len(), 1);
        assert_eq!(data.log[0].summary, "Did a thing");
    }

    #[test]
    fn test_load_missing_state_uses_default() {
        let yaml = "items: []\nlog: []\n";
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let data = load(f.path()).unwrap();
        assert!(data.state.branch.is_empty());
    }
}
