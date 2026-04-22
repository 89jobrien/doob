# doobdash Multi-Repo Handoff Index — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform doobdash from a single-file viewer into a workspace-wide kanban
dashboard that merges handoff items from all active repos, filterable by project.

**Architecture:** Hexagonal architecture with three ports (`RegistrySource`,
`HandoffDiscovery`, `HandoffLoader`) and three filesystem adapters. `Workspace` is
a pure domain type with no I/O. `App` holds a `Workspace` instead of a single
`HandoffData`. Composition root in `main.rs` wires adapters → domain → TUI.

**Tech Stack:** Rust 2021, ratatui 0.29, crossterm 0.28, serde/serde_yaml, toml,
rayon (parallel load), anyhow, tempfile (tests)

---

## File Map

| Action | File                               | Responsibility                                                                                            |
| ------ | ---------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Create | `crates/doobdash/src/discovery.rs` | Ports + adapters: `RegistrySource`, `HandoffDiscovery`, `HandoffLoader`, all three adapters, test doubles |
| Modify | `crates/doobdash/src/data.rs`      | Add `SourceRef` to `HandoffData`; add optional top-level YAML fields; keep existing API                   |
| Create | `crates/doobdash/src/workspace.rs` | Pure domain: `Workspace` struct, `visible_items()`, `filtered_items()`, stats helpers                     |
| Modify | `crates/doobdash/src/app.rs`       | Replace `data: HandoffData` with `workspace: Workspace`; add `Mode::Filter`; update all helpers           |
| Modify | `crates/doobdash/src/main.rs`      | Wire adapters in composition root; replace single `data::load()` call; add filter key handlers            |
| Modify | `crates/doobdash/src/ui.rs`        | Header badge, card repo tags, filter picker overlay, per-project stats table                              |
| Modify | `crates/doobdash/Cargo.toml`       | Add `toml = "0.8"` and `rayon = "1"` dependencies                                                         |

---

## Task 1: Add `toml` and `rayon` dependencies

**Files:**

- Modify: `crates/doobdash/Cargo.toml`

- [ ] **Step 1: Add dependencies**

Open `crates/doobdash/Cargo.toml`. In the `[dependencies]` section, add:

```toml
toml = "0.8"
rayon = "1"
```

- [ ] **Step 2: Verify it compiles**

```bash
cd /Users/joe/dev/doob && cargo check -p doobdash
```

Expected: no errors (warnings about unused imports are fine at this stage).

- [ ] **Step 3: Commit**

```bash
git add crates/doobdash/Cargo.toml
git commit -m "chore(doobdash): add toml and rayon deps"
```

---

## Task 2: Add `SourceRef` to `data.rs` and enrich YAML loading

**Files:**

- Modify: `crates/doobdash/src/data.rs`

- [ ] **Step 1: Write a failing test for `SourceRef` presence on loaded data**

Add to the `#[cfg(test)]` block at the bottom of `data.rs`:

```rust
#[test]
fn test_handoff_data_has_source_ref() {
    let yaml = "items: []\nlog: []\n";
    let mut f = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    f.write_all(yaml.as_bytes()).unwrap();
    let source = SourceRef {
        name: "myrepo".to_string(),
        path: f.path().to_path_buf(),
        tags: vec!["rust".to_string()],
        url: None,
    };
    let data = load(f.path(), source.clone()).unwrap();
    assert_eq!(data.source.name, "myrepo");
    assert_eq!(data.source.tags, vec!["rust"]);
    // handoff_path kept for compat
    assert_eq!(data.handoff_path, f.path());
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash test_handoff_data_has_source_ref 2>&1 | tail -20
```

Expected: compile error — `SourceRef` not defined, `load` has wrong signature.

- [ ] **Step 3: Implement `SourceRef` and update `HandoffData` and `load`**

Replace the entire contents of `crates/doobdash/src/data.rs` with:

```rust
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExtraEntry {
    #[serde(default)]
    pub date: String,
    #[serde(default, rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub note: String,
}

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
    #[serde(default)]
    pub extra: Vec<ExtraEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LogEntry {
    pub date: String,
    pub summary: String,
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
    #[allow(dead_code)]
    pub notes: String,
}

/// Identity of the source HANDOFF file for a loaded dataset.
#[derive(Debug, Clone, Default)]
pub struct SourceRef {
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HandoffData {
    pub source: SourceRef,
    pub items: Vec<YamlItem>,
    pub log: Vec<LogEntry>,
    pub state: StateData,
    /// Kept for backward compatibility; equals `source.path`.
    pub handoff_path: PathBuf,
}

/// Top-level optional YAML fields for project identity.
#[derive(Debug, Deserialize, Default)]
struct YamlMeta {
    #[serde(default)]
    project: String,
    #[serde(default)]
    url: String,
}

/// Load a HANDOFF YAML file, attaching the provided `SourceRef`.
///
/// If `source.name` is empty, it is inferred from the parent directory name.
pub fn load(handoff_path: &Path, mut source: SourceRef) -> Result<HandoffData> {
    let raw = std::fs::read_to_string(handoff_path)?;
    let val: serde_yaml::Value = serde_yaml::from_str(&raw)?;

    // Parse optional top-level identity fields
    let meta: YamlMeta = serde_yaml::from_value(val.clone()).unwrap_or_default();

    // Resolve display name: source.name wins, then YAML project, then dirname
    if source.name.is_empty() {
        source.name = if !meta.project.is_empty() {
            meta.project.clone()
        } else {
            handoff_path
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown".to_string())
        };
    }

    // Resolve URL: source wins, then YAML
    if source.url.is_none() && !meta.url.is_empty() {
        source.url = Some(meta.url);
    }

    source.path = handoff_path.to_path_buf();

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
        handoff_path: source.path.clone(),
        source,
        items,
        log,
        state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_source() -> SourceRef {
        SourceRef::default()
    }

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
        let data = load(f.path(), make_source()).unwrap();
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
        let data = load(f.path(), make_source()).unwrap();
        assert!(data.state.branch.is_empty());
    }

    #[test]
    fn test_load_extra_entries() {
        let yaml = r#"
items:
- id: doob-2
  priority: P0
  status: blocked
  title: Blocked item
  extra:
  - date: "2026-04-01"
    type: note
    note: "This is blocked by X"
"#;
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let data = load(f.path(), make_source()).unwrap();
        assert_eq!(data.items[0].extra.len(), 1);
        assert_eq!(data.items[0].extra[0].note, "This is blocked by X");
    }

    #[test]
    fn test_handoff_data_has_source_ref() {
        let yaml = "items: []\nlog: []\n";
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let source = SourceRef {
            name: "myrepo".to_string(),
            path: f.path().to_path_buf(),
            tags: vec!["rust".to_string()],
            url: None,
        };
        let data = load(f.path(), source.clone()).unwrap();
        assert_eq!(data.source.name, "myrepo");
        assert_eq!(data.source.tags, vec!["rust"]);
        assert_eq!(data.handoff_path, f.path());
    }

    #[test]
    fn test_name_inferred_from_yaml_project_field() {
        let yaml = "project: mything\nitems: []\nlog: []\n";
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        // source.name is empty — should fall through to YAML project field
        let data = load(f.path(), SourceRef::default()).unwrap();
        assert_eq!(data.source.name, "mything");
    }

    #[test]
    fn test_registry_name_beats_yaml_project() {
        let yaml = "project: yaml-name\nitems: []\nlog: []\n";
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let source = SourceRef { name: "registry-name".to_string(), ..Default::default() };
        let data = load(f.path(), source).unwrap();
        assert_eq!(data.source.name, "registry-name");
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/doobdash/src/data.rs
git commit -m "feat(doobdash): add SourceRef to HandoffData, enrich YAML load"
```

---

## Task 3: Create `discovery.rs` — ports, adapters, test doubles

**Files:**

- Create: `crates/doobdash/src/discovery.rs`

- [ ] **Step 1: Write failing tests for `RegistryConfig` parsing**

Create `crates/doobdash/src/discovery.rs` with only the test module:

```rust
// crates/doobdash/src/discovery.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_registry_with_repos() {
        let toml_src = r#"
scan_root = "/home/user/dev"

[[repo]]
name = "alpha"
path = "/home/user/dev/alpha"
tags = ["rust"]

[[repo]]
name = "beta"
path = "/home/user/dev/beta"
tags = []
"#;
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(toml_src.as_bytes()).unwrap();
        let src = TomlRegistrySource { path: f.path().to_path_buf() };
        let config = src.load().unwrap();
        assert_eq!(config.scan_root, std::path::PathBuf::from("/home/user/dev"));
        assert_eq!(config.repos.len(), 2);
        assert_eq!(config.repos[0].name, "alpha");
        assert_eq!(config.repos[1].tags, Vec::<String>::new());
    }

    #[test]
    fn test_missing_registry_returns_default() {
        let src = TomlRegistrySource {
            path: std::path::PathBuf::from("/nonexistent/handoffs.toml"),
        };
        let config = src.load().unwrap();
        assert!(config.repos.is_empty());
        // scan_root should default to ~/dev
        assert!(config.scan_root.ends_with("dev"));
    }

    #[test]
    fn test_fs_discovery_finds_handoff_files() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let repo_dir = dir.path().join("myrepo");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(repo_dir.join("HANDOFF.myrepo.yaml"), "items: []").unwrap();
        // Should be skipped
        fs::write(repo_dir.join("HANDOFF.state.yaml"), "branch: main").unwrap();

        let config = RegistryConfig {
            scan_root: dir.path().to_path_buf(),
            repos: vec![],
        };
        let discovery = FsHandoffDiscovery;
        let paths = discovery.discover(&config).unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("HANDOFF.myrepo.yaml"));
    }

    #[test]
    fn test_fs_discovery_skips_target_and_git() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        // Files inside skipped dirs should not appear
        let target_dir = dir.path().join("target").join("sub");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("HANDOFF.ignore.yaml"), "items: []").unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("HANDOFF.ignore.yaml"), "items: []").unwrap();

        let config = RegistryConfig {
            scan_root: dir.path().to_path_buf(),
            repos: vec![],
        };
        let paths = FsHandoffDiscovery.discover(&config).unwrap();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_registry_entries_merged_with_discovery() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        // A registered repo (explicit)
        let reg_dir = dir.path().join("registered");
        fs::create_dir_all(&reg_dir).unwrap();
        let reg_file = reg_dir.join("HANDOFF.reg.yaml");
        fs::write(&reg_file, "items: []").unwrap();
        // A discovered repo (not in registry)
        let disc_dir = dir.path().join("discovered");
        fs::create_dir_all(&disc_dir).unwrap();
        fs::write(disc_dir.join("HANDOFF.disc.yaml"), "items: []").unwrap();

        let config = RegistryConfig {
            scan_root: dir.path().to_path_buf(),
            repos: vec![RepoEntry {
                name: "registered".to_string(),
                path: reg_dir.clone(),
                tags: vec!["explicit".to_string()],
                url: None,
            }],
        };
        let discovery = FsHandoffDiscovery;
        let paths = discovery.discover(&config).unwrap();
        // Both should appear
        assert_eq!(paths.len(), 2);
    }
}
```

- [ ] **Step 2: Run to verify compile failure**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | head -30
```

Expected: compile errors — types not yet defined.

- [ ] **Step 3: Implement `discovery.rs`**

Replace the file with the full implementation:

```rust
// crates/doobdash/src/discovery.rs
//
// Ports (traits) and adapters for registry loading and HANDOFF file discovery.
// Domain code (Workspace) depends on these traits, not the concrete adapters.

use crate::data::SourceRef;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// A single entry in `handoffs.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RepoEntry {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// Parsed content of `handoffs.toml`.
#[derive(Debug, Clone, Default)]
pub struct RegistryConfig {
    pub scan_root: PathBuf,
    pub repos: Vec<RepoEntry>,
}

// ---------------------------------------------------------------------------
// Ports (traits)
// ---------------------------------------------------------------------------

pub trait RegistrySource {
    fn load(&self) -> Result<RegistryConfig>;
}

pub trait HandoffDiscovery {
    /// Returns canonical paths to every HANDOFF.*.yaml to load.
    /// Merges registry entries with auto-discovered files; registry wins on conflict.
    fn discover(&self, config: &RegistryConfig) -> Result<Vec<PathBuf>>;
}

// ---------------------------------------------------------------------------
// Adapters
// ---------------------------------------------------------------------------

/// Reads `~/.ctx/doob/handoffs.toml`. Returns default if missing.
pub struct TomlRegistrySource {
    pub path: PathBuf,
}

impl TomlRegistrySource {
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join(".ctx/doob/handoffs.toml")
    }
}

#[derive(Deserialize, Default)]
struct RawRegistry {
    #[serde(default)]
    scan_root: Option<String>,
    #[serde(default)]
    repo: Vec<RepoEntry>,
}

impl RegistrySource for TomlRegistrySource {
    fn load(&self) -> Result<RegistryConfig> {
        if !self.path.exists() {
            let scan_root = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/"))
                .join("dev");
            return Ok(RegistryConfig { scan_root, repos: vec![] });
        }
        let raw = std::fs::read_to_string(&self.path)?;
        let parsed: RawRegistry = toml::from_str(&raw)?;
        let scan_root = parsed
            .scan_root
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/"))
                    .join("dev")
            });
        Ok(RegistryConfig { scan_root, repos: parsed.repo })
    }
}

/// Walks `scan_root` and merges with registry entries.
pub struct FsHandoffDiscovery;

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", ".worktrees"];

impl HandoffDiscovery for FsHandoffDiscovery {
    fn discover(&self, config: &RegistryConfig) -> Result<Vec<PathBuf>> {
        // Collect registry paths first (explicit, win on conflict)
        let mut seen: HashSet<PathBuf> = HashSet::new();
        let mut result: Vec<PathBuf> = vec![];

        for repo in &config.repos {
            if let Ok(handoff) = find_handoff_in_dir(&repo.path) {
                let canonical = handoff.canonicalize().unwrap_or_else(|_| handoff.clone());
                if seen.insert(canonical.clone()) {
                    result.push(canonical);
                }
            }
        }

        // Walk scan_root for any not already in registry
        walk_for_handoffs(&config.scan_root, &mut seen, &mut result);

        Ok(result)
    }
}

fn find_handoff_in_dir(dir: &Path) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)?.flatten() {
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if s.starts_with("HANDOFF.")
            && s.ends_with(".yaml")
            && s != "HANDOFF.state.yaml"
        {
            return Ok(entry.path());
        }
    }
    anyhow::bail!("no HANDOFF file in {}", dir.display())
}

fn walk_for_handoffs(dir: &Path, seen: &mut HashSet<PathBuf>, result: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let s = name.to_string_lossy();

        if path.is_dir() {
            if SKIP_DIRS.contains(&s.as_ref()) {
                continue;
            }
            walk_for_handoffs(&path, seen, result);
        } else if s.starts_with("HANDOFF.")
            && s.ends_with(".yaml")
            && s != "HANDOFF.state.yaml"
        {
            let canonical = path.canonicalize().unwrap_or(path);
            if seen.insert(canonical.clone()) {
                result.push(canonical);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SourceRef builder — converts a RepoEntry to a SourceRef
// ---------------------------------------------------------------------------

pub fn source_ref_for_path(path: &Path, config: &RegistryConfig) -> SourceRef {
    // Check if this path is under a registered repo
    for repo in &config.repos {
        if path.starts_with(&repo.path) {
            return SourceRef {
                name: repo.name.clone(),
                path: path.to_path_buf(),
                tags: repo.tags.clone(),
                url: repo.url.clone(),
            };
        }
    }
    // Not in registry — infer name from parent dir
    let name = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());
    SourceRef { name, path: path.to_path_buf(), tags: vec![], url: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_registry_with_repos() {
        let toml_src = r#"
scan_root = "/home/user/dev"

[[repo]]
name = "alpha"
path = "/home/user/dev/alpha"
tags = ["rust"]

[[repo]]
name = "beta"
path = "/home/user/dev/beta"
tags = []
"#;
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(toml_src.as_bytes()).unwrap();
        let src = TomlRegistrySource { path: f.path().to_path_buf() };
        let config = src.load().unwrap();
        assert_eq!(config.scan_root, PathBuf::from("/home/user/dev"));
        assert_eq!(config.repos.len(), 2);
        assert_eq!(config.repos[0].name, "alpha");
        assert_eq!(config.repos[1].tags, Vec::<String>::new());
    }

    #[test]
    fn test_missing_registry_returns_default() {
        let src = TomlRegistrySource {
            path: PathBuf::from("/nonexistent/handoffs.toml"),
        };
        let config = src.load().unwrap();
        assert!(config.repos.is_empty());
        assert!(config.scan_root.ends_with("dev"));
    }

    #[test]
    fn test_fs_discovery_finds_handoff_files() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let repo_dir = dir.path().join("myrepo");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(repo_dir.join("HANDOFF.myrepo.yaml"), "items: []").unwrap();
        fs::write(repo_dir.join("HANDOFF.state.yaml"), "branch: main").unwrap();

        let config = RegistryConfig {
            scan_root: dir.path().to_path_buf(),
            repos: vec![],
        };
        let paths = FsHandoffDiscovery.discover(&config).unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].to_string_lossy().contains("HANDOFF.myrepo.yaml"));
    }

    #[test]
    fn test_fs_discovery_skips_target_and_git() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let target_dir = dir.path().join("target").join("sub");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("HANDOFF.ignore.yaml"), "items: []").unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("HANDOFF.ignore.yaml"), "items: []").unwrap();

        let config = RegistryConfig {
            scan_root: dir.path().to_path_buf(),
            repos: vec![],
        };
        let paths = FsHandoffDiscovery.discover(&config).unwrap();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_registry_entries_merged_with_discovery() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let reg_dir = dir.path().join("registered");
        fs::create_dir_all(&reg_dir).unwrap();
        let reg_file = reg_dir.join("HANDOFF.reg.yaml");
        fs::write(&reg_file, "items: []").unwrap();
        let disc_dir = dir.path().join("discovered");
        fs::create_dir_all(&disc_dir).unwrap();
        fs::write(disc_dir.join("HANDOFF.disc.yaml"), "items: []").unwrap();

        let config = RegistryConfig {
            scan_root: dir.path().to_path_buf(),
            repos: vec![RepoEntry {
                name: "registered".to_string(),
                path: reg_dir.clone(),
                tags: vec!["explicit".to_string()],
                url: None,
            }],
        };
        let paths = FsHandoffDiscovery.discover(&config).unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_source_ref_resolves_from_registry() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().join("myrepo");
        let handoff = repo_path.join("HANDOFF.myrepo.yaml");
        let config = RegistryConfig {
            scan_root: dir.path().to_path_buf(),
            repos: vec![RepoEntry {
                name: "myrepo".to_string(),
                path: repo_path.clone(),
                tags: vec!["rust".to_string()],
                url: Some("https://example.com".to_string()),
            }],
        };
        let sref = source_ref_for_path(&handoff, &config);
        assert_eq!(sref.name, "myrepo");
        assert_eq!(sref.tags, vec!["rust"]);
        assert_eq!(sref.url, Some("https://example.com".to_string()));
    }
}
```

- [ ] **Step 4: Add `dirs` crate dependency** (needed for `dirs::home_dir`)

In `crates/doobdash/Cargo.toml`, add under `[dependencies]`:

```toml
dirs = "5"
```

- [ ] **Step 5: Register module in `main.rs`**

At the top of `crates/doobdash/src/main.rs`, add:

```rust
mod discovery;
```

alongside the existing `mod data;` line.

- [ ] **Step 6: Run tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/doobdash/src/discovery.rs crates/doobdash/src/main.rs crates/doobdash/Cargo.toml
git commit -m "feat(doobdash): add discovery ports, adapters, registry loading"
```

---

## Task 4: Create `workspace.rs` — pure domain type

**Files:**

- Create: `crates/doobdash/src/workspace.rs`

- [ ] **Step 1: Write failing tests**

Create `crates/doobdash/src/workspace.rs` with only the test block:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{HandoffData, SourceRef, YamlItem};

    fn make_item(id: &str, status: &str, source_name: &str) -> (YamlItem, String) {
        (
            YamlItem { id: id.to_string(), status: status.to_string(), title: id.to_string(), ..Default::default() },
            source_name.to_string(),
        )
    }

    fn make_source(name: &str) -> HandoffData {
        HandoffData {
            source: SourceRef { name: name.to_string(), ..Default::default() },
            ..Default::default()
        }
    }

    fn make_workspace_with(sources: Vec<(&str, Vec<(&str, &str)>)>) -> Workspace {
        let data = sources.into_iter().map(|(name, items)| {
            let mut hd = make_source(name);
            hd.items = items.into_iter().map(|(id, status)| YamlItem {
                id: id.to_string(),
                status: status.to_string(),
                title: id.to_string(),
                ..Default::default()
            }).collect();
            hd
        }).collect();
        Workspace::new(data)
    }

    #[test]
    fn test_visible_items_all_when_no_filter() {
        let ws = make_workspace_with(vec![
            ("repo-a", vec![("a1", "open"), ("a2", "done")]),
            ("repo-b", vec![("b1", "open")]),
        ]);
        assert_eq!(ws.visible_items().len(), 3);
    }

    #[test]
    fn test_visible_items_filtered_by_project() {
        let mut ws = make_workspace_with(vec![
            ("repo-a", vec![("a1", "open"), ("a2", "done")]),
            ("repo-b", vec![("b1", "open")]),
        ]);
        ws.active_filter = Some("repo-a".to_string());
        assert_eq!(ws.visible_items().len(), 2);
        assert!(ws.visible_items().iter().all(|(_, s)| s.name == "repo-a"));
    }

    #[test]
    fn test_project_names_sorted() {
        let ws = make_workspace_with(vec![
            ("zebra", vec![]),
            ("alpha", vec![]),
        ]);
        let names = ws.project_names();
        assert_eq!(names, vec!["alpha", "zebra"]);
    }

    #[test]
    fn test_per_project_stats() {
        let ws = make_workspace_with(vec![
            ("repo-a", vec![("a1", "open"), ("a2", "done"), ("a3", "blocked")]),
            ("repo-b", vec![("b1", "done")]),
        ]);
        let stats = ws.per_project_stats();
        let a = stats.iter().find(|s| s.name == "repo-a").unwrap();
        assert_eq!(a.open, 1);
        assert_eq!(a.done, 1);
        assert_eq!(a.blocked, 1);
        let b = stats.iter().find(|s| s.name == "repo-b").unwrap();
        assert_eq!(b.done, 1);
    }

    #[test]
    fn test_source_ref_attached_to_visible_items() {
        let ws = make_workspace_with(vec![
            ("myrepo", vec![("x1", "open")]),
        ]);
        let items = ws.visible_items();
        assert_eq!(items[0].1.name, "myrepo");
    }
}
```

- [ ] **Step 2: Run to verify compile failure**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash workspace 2>&1 | head -20
```

Expected: compile errors — `Workspace` not defined.

- [ ] **Step 3: Implement `workspace.rs`**

Replace the file contents:

```rust
// crates/doobdash/src/workspace.rs
//
// Pure domain type. No filesystem I/O. Holds loaded HandoffData sources and
// provides the merged/filtered view consumed by App and UI.

use crate::data::{HandoffData, SourceRef, YamlItem};

/// Per-project item counts for the Stats tab.
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub name: String,
    pub open: usize,
    pub blocked: usize,
    pub done: usize,
    pub parked: usize,
}

pub struct Workspace {
    pub sources: Vec<HandoffData>,
    /// None = show all projects. Some(name) = show only that project.
    pub active_filter: Option<String>,
}

impl Workspace {
    pub fn new(sources: Vec<HandoffData>) -> Self {
        Workspace { sources, active_filter: None }
    }

    /// All items across all sources with their source ref attached.
    pub fn items(&self) -> Vec<(&YamlItem, &SourceRef)> {
        self.sources
            .iter()
            .flat_map(|hd| hd.items.iter().map(move |item| (item, &hd.source)))
            .collect()
    }

    /// Effective item list — respects `active_filter`.
    pub fn visible_items(&self) -> Vec<(&YamlItem, &SourceRef)> {
        match &self.active_filter {
            None => self.items(),
            Some(filter) => self
                .sources
                .iter()
                .filter(|hd| &hd.source.name == filter)
                .flat_map(|hd| hd.items.iter().map(move |item| (item, &hd.source)))
                .collect(),
        }
    }

    /// Sorted list of all known project names.
    pub fn project_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.sources.iter().map(|hd| hd.source.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    /// Per-project item counts for the Stats tab.
    pub fn per_project_stats(&self) -> Vec<ProjectStats> {
        let mut stats: Vec<ProjectStats> = self
            .sources
            .iter()
            .map(|hd| {
                let open = hd.items.iter().filter(|i| matches!(i.status.as_str(), "open" | "in-progress")).count();
                let blocked = hd.items.iter().filter(|i| i.status == "blocked").count();
                let done = hd.items.iter().filter(|i| i.status == "done").count();
                let parked = hd.items.iter().filter(|i| matches!(i.status.as_str(), "parked" | "waiting")).count();
                ProjectStats { name: hd.source.name.clone(), open, blocked, done, parked }
            })
            .collect();
        stats.sort_by(|a, b| a.name.cmp(&b.name));
        stats
    }

    /// Total counts across all visible items.
    pub fn active_count(&self) -> usize {
        self.visible_items().iter().filter(|(i, _)| matches!(i.status.as_str(), "open" | "blocked" | "in-progress")).count()
    }

    pub fn waiting_count(&self) -> usize {
        self.visible_items().iter().filter(|(i, _)| matches!(i.status.as_str(), "parked" | "waiting")).count()
    }

    pub fn done_count(&self) -> usize {
        self.visible_items().iter().filter(|(i, _)| i.status == "done").count()
    }

    /// The first source that contains the item with the given id.
    pub fn source_for_item_id(&self, id: &str) -> Option<&HandoffData> {
        self.sources.iter().find(|hd| hd.items.iter().any(|i| i.id == id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{HandoffData, SourceRef, YamlItem};

    fn make_workspace_with(sources: Vec<(&str, Vec<(&str, &str)>)>) -> Workspace {
        let data = sources.into_iter().map(|(name, items)| {
            let mut hd = HandoffData {
                source: SourceRef { name: name.to_string(), ..Default::default() },
                ..Default::default()
            };
            hd.items = items.into_iter().map(|(id, status)| YamlItem {
                id: id.to_string(),
                status: status.to_string(),
                title: id.to_string(),
                ..Default::default()
            }).collect();
            hd
        }).collect();
        Workspace::new(data)
    }

    #[test]
    fn test_visible_items_all_when_no_filter() {
        let ws = make_workspace_with(vec![
            ("repo-a", vec![("a1", "open"), ("a2", "done")]),
            ("repo-b", vec![("b1", "open")]),
        ]);
        assert_eq!(ws.visible_items().len(), 3);
    }

    #[test]
    fn test_visible_items_filtered_by_project() {
        let mut ws = make_workspace_with(vec![
            ("repo-a", vec![("a1", "open"), ("a2", "done")]),
            ("repo-b", vec![("b1", "open")]),
        ]);
        ws.active_filter = Some("repo-a".to_string());
        assert_eq!(ws.visible_items().len(), 2);
        assert!(ws.visible_items().iter().all(|(_, s)| s.name == "repo-a"));
    }

    #[test]
    fn test_project_names_sorted() {
        let ws = make_workspace_with(vec![
            ("zebra", vec![]),
            ("alpha", vec![]),
        ]);
        let names = ws.project_names();
        assert_eq!(names, vec!["alpha", "zebra"]);
    }

    #[test]
    fn test_per_project_stats() {
        let ws = make_workspace_with(vec![
            ("repo-a", vec![("a1", "open"), ("a2", "done"), ("a3", "blocked")]),
            ("repo-b", vec![("b1", "done")]),
        ]);
        let stats = ws.per_project_stats();
        let a = stats.iter().find(|s| s.name == "repo-a").unwrap();
        assert_eq!(a.open, 1);
        assert_eq!(a.done, 1);
        assert_eq!(a.blocked, 1);
        let b = stats.iter().find(|s| s.name == "repo-b").unwrap();
        assert_eq!(b.done, 1);
    }

    #[test]
    fn test_source_ref_attached_to_visible_items() {
        let ws = make_workspace_with(vec![
            ("myrepo", vec![("x1", "open")]),
        ]);
        let items = ws.visible_items();
        assert_eq!(items[0].1.name, "myrepo");
    }
}
```

- [ ] **Step 4: Register module in `main.rs`**

Add `mod workspace;` alongside `mod data;` in `main.rs`.

- [ ] **Step 5: Run tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash workspace 2>&1 | tail -20
```

Expected: all workspace tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/doobdash/src/workspace.rs crates/doobdash/src/main.rs
git commit -m "feat(doobdash): add Workspace pure domain type"
```

---

## Task 5: Migrate `app.rs` to `Workspace`

**Files:**

- Modify: `crates/doobdash/src/app.rs`

- [ ] **Step 1: Add `Mode::Filter` and `filter_selected` field — write test first**

Add to the test block in `app.rs`:

```rust
#[test]
fn test_filter_mode_set_and_clear() {
    use crate::workspace::Workspace;
    let app = App::new(Workspace::new(vec![]));
    assert_eq!(app.mode, Mode::Normal);
    // filter_selected starts at 0
    assert_eq!(app.filter_selected, 0);
}
```

- [ ] **Step 2: Run to verify compile failure**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash test_filter_mode 2>&1 | head -20
```

Expected: compile error — `App::new` still takes `HandoffData`.

- [ ] **Step 3: Replace `app.rs`**

Replace the entire file with the updated version that swaps `data: HandoffData` for
`workspace: Workspace`, adds `Mode::Filter`, adds `filter_selected: usize`, and
delegates count helpers to `Workspace`. All existing tests must continue to pass —
adapt the test helpers to build a `Workspace` instead of `HandoffData`:

```rust
mod actions;
// (keep existing imports, add workspace)
use crate::data::YamlItem; // still needed for test helpers
use crate::workspace::Workspace;
use crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    InputNote,
    PickStatus,
    Search,
    Overlay,
    Filter, // NEW: project picker
}

// StripState, Tab, Column — unchanged (copy verbatim from existing file)

pub struct App {
    pub workspace: Workspace,  // replaces `data: HandoffData`
    pub selected: usize,
    pub mode: Mode,
    pub input_buf: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub should_save: bool,
    pub active_tab: Tab,
    pub active_col: Column,
    pub search_query: String,
    pub col_selected: [usize; 3],
    pub col_offsets: [usize; 3],
    pub last_key: Option<KeyCode>,
    pub strip: StripState,
    pub overlay_scroll: usize,
    pub filter_selected: usize, // NEW: index into project_names() in Filter mode
}

impl App {
    pub fn new(workspace: Workspace) -> Self {
        App {
            workspace,
            selected: 0,
            mode: Mode::Normal,
            input_buf: String::new(),
            status_message: None,
            should_quit: false,
            should_save: false,
            active_tab: Tab::Items,
            active_col: Column::Active,
            search_query: String::new(),
            col_selected: [0; 3],
            col_offsets: [0; 3],
            last_key: None,
            strip: StripState::default(),
            overlay_scroll: 0,
            filter_selected: 0,
        }
    }
    // ... all existing navigation methods, but col_items() now uses
    // workspace.visible_items() instead of data.items
}
```

The full replacement is provided below. Copy this exactly:

```rust
use crate::workspace::Workspace;
use crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    InputNote,
    PickStatus,
    Search,
    Overlay,
    Filter,
}

#[derive(Debug)]
pub struct StripState {
    pub visible: bool,
    pub height: u16,
    pub z_held_since: Option<std::time::Instant>,
}

impl Clone for StripState {
    fn clone(&self) -> Self {
        StripState { visible: self.visible, height: self.height, z_held_since: None }
    }
}

impl Default for StripState {
    fn default() -> Self {
        StripState { visible: true, height: 3, z_held_since: None }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Tab {
    Items,
    Log,
    Stats,
    Help,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Column {
    Active,
    Waiting,
    Done,
}

impl Column {
    pub fn index(self) -> usize {
        match self {
            Column::Active => 0,
            Column::Waiting => 1,
            Column::Done => 2,
        }
    }
    pub fn next(self) -> Self {
        match self {
            Column::Active => Column::Waiting,
            Column::Waiting => Column::Done,
            Column::Done => Column::Done,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Column::Active => Column::Active,
            Column::Waiting => Column::Active,
            Column::Done => Column::Waiting,
        }
    }
    pub fn matches_status(self, status: &str) -> bool {
        match self {
            Column::Active => matches!(status, "open" | "blocked" | "in-progress"),
            Column::Waiting => matches!(status, "parked" | "waiting"),
            Column::Done => status == "done",
        }
    }
}

pub struct App {
    pub workspace: Workspace,
    pub selected: usize,
    pub mode: Mode,
    pub input_buf: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub should_save: bool,
    pub active_tab: Tab,
    pub active_col: Column,
    pub search_query: String,
    pub col_selected: [usize; 3],
    pub col_offsets: [usize; 3],
    pub last_key: Option<KeyCode>,
    pub strip: StripState,
    pub overlay_scroll: usize,
    pub filter_selected: usize,
}

impl App {
    pub fn new(workspace: Workspace) -> Self {
        App {
            workspace,
            selected: 0,
            mode: Mode::Normal,
            input_buf: String::new(),
            status_message: None,
            should_quit: false,
            should_save: false,
            active_tab: Tab::Items,
            active_col: Column::Active,
            search_query: String::new(),
            col_selected: [0; 3],
            col_offsets: [0; 3],
            last_key: None,
            strip: StripState::default(),
            overlay_scroll: 0,
            filter_selected: 0,
        }
    }

    /// Items visible in `col` after applying search filter.
    pub fn col_items(&self, col: Column) -> Vec<usize> {
        let q = self.search_query.to_lowercase();
        self.workspace
            .visible_items()
            .into_iter()
            .enumerate()
            .filter(|(_, (item, _))| col.matches_status(&item.status))
            .filter(|(_, (item, _))| {
                q.is_empty()
                    || item.title.to_lowercase().contains(&q)
                    || item.id.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn current_col_items(&self) -> Vec<usize> {
        self.col_items(self.active_col)
    }

    /// Index of the currently selected item in visible_items(), if any.
    pub fn selected_item_index(&self) -> Option<usize> {
        let items = self.current_col_items();
        let sel = self.col_selected[self.active_col.index()];
        items.get(sel).copied()
    }

    pub fn select_next(&mut self) {
        let col = self.active_col.index();
        let len = self.col_items(self.active_col).len();
        if len > 0 {
            self.col_selected[col] = (self.col_selected[col] + 1).min(len - 1);
        }
        self.sync_legacy_selected();
    }

    pub fn select_prev(&mut self) {
        let col = self.active_col.index();
        let len = self.col_items(self.active_col).len();
        if len > 0 {
            self.col_selected[col] = self.col_selected[col].saturating_sub(1);
        }
        self.sync_legacy_selected();
    }

    pub fn select_top(&mut self) {
        self.col_selected[self.active_col.index()] = 0;
        self.col_offsets[self.active_col.index()] = 0;
        self.sync_legacy_selected();
    }

    pub fn select_bottom(&mut self) {
        let col = self.active_col.index();
        let len = self.col_items(self.active_col).len();
        if len > 0 {
            self.col_selected[col] = len - 1;
        }
        self.sync_legacy_selected();
    }

    pub fn col_next(&mut self) {
        self.active_col = self.active_col.next();
        self.sync_legacy_selected();
    }

    pub fn col_prev(&mut self) {
        self.active_col = self.active_col.prev();
        self.sync_legacy_selected();
    }

    fn sync_legacy_selected(&mut self) {
        if let Some(idx) = self.selected_item_index() {
            self.selected = idx;
        }
    }

    pub fn selected_id(&self) -> Option<String> {
        self.selected_item_index()
            .and_then(|i| self.workspace.visible_items().get(i).map(|(item, _)| item.id.clone()))
    }

    pub fn active_count(&self) -> usize {
        self.workspace.active_count()
    }

    pub fn waiting_count(&self) -> usize {
        self.workspace.waiting_count()
    }

    pub fn done_count(&self) -> usize {
        self.workspace.done_count()
    }

    pub fn strip_toggle(&mut self) { self.strip.visible = !self.strip.visible; }
    pub fn strip_expand(&mut self) { self.strip.height += 1; }
    pub fn strip_shrink(&mut self) { if self.strip.height > 1 { self.strip.height -= 1; } }

    pub fn z_is_held(&self) -> bool {
        self.strip.z_held_since.map(|t| t.elapsed().as_millis() < 1000).unwrap_or(false)
    }
    pub fn z_press(&mut self) { self.strip.z_held_since = Some(std::time::Instant::now()); }
    pub fn z_release(&mut self) { self.strip.z_held_since = None; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{HandoffData, SourceRef, YamlItem};
    use crate::workspace::Workspace;

    fn make_app_with_statuses(statuses: &[&str]) -> App {
        let items = statuses.iter().enumerate().map(|(i, s)| YamlItem {
            id: format!("item-{i}"),
            priority: "P1".into(),
            status: s.to_string(),
            title: format!("Item {i}"),
            ..Default::default()
        }).collect();
        let hd = HandoffData { items, ..Default::default() };
        App::new(Workspace::new(vec![hd]))
    }

    fn make_app(n: usize) -> App {
        let statuses: Vec<&str> = (0..n).map(|_| "open").collect();
        make_app_with_statuses(&statuses)
    }

    #[test]
    fn test_select_wraps() {
        let mut app = make_app(3);
        app.col_selected[0] = 2;
        app.select_next();
        assert_eq!(app.col_selected[0], 2);
    }

    #[test]
    fn test_select_prev_wraps() {
        let mut app = make_app(3);
        app.col_selected[0] = 0;
        app.select_prev();
        assert_eq!(app.col_selected[0], 0);
    }

    #[test]
    fn test_selected_id() {
        let app = make_app(2);
        assert_eq!(app.selected_id(), Some("item-0".to_string()));
    }

    #[test]
    fn test_col_items_filter_by_status() {
        let app = make_app_with_statuses(&["open", "done", "parked", "open"]);
        assert_eq!(app.col_items(Column::Active).len(), 2);
        assert_eq!(app.col_items(Column::Done).len(), 1);
        assert_eq!(app.col_items(Column::Waiting).len(), 1);
    }

    #[test]
    fn test_search_filter() {
        let mut app = make_app_with_statuses(&["open", "open"]);
        // Patch titles via workspace
        app.workspace.sources[0].items[0].title = "foo bar".into();
        app.workspace.sources[0].items[1].title = "baz qux".into();
        app.search_query = "foo".into();
        assert_eq!(app.col_items(Column::Active).len(), 1);
    }

    #[test]
    fn test_col_nav() {
        let mut app = make_app(1);
        assert_eq!(app.active_col, Column::Active);
        app.col_next();
        assert_eq!(app.active_col, Column::Waiting);
        app.col_prev();
        assert_eq!(app.active_col, Column::Active);
    }

    #[test]
    fn test_counts() {
        let app = make_app_with_statuses(&["open", "done", "parked", "blocked"]);
        assert_eq!(app.active_count(), 2);
        assert_eq!(app.waiting_count(), 1);
        assert_eq!(app.done_count(), 1);
    }

    #[test]
    fn test_strip_default_height() {
        let app = make_app(1);
        assert_eq!(app.strip.height, 3);
        assert!(app.strip.visible);
    }

    #[test]
    fn test_strip_toggle() {
        let mut app = make_app(1);
        app.strip_toggle();
        assert!(!app.strip.visible);
        app.strip_toggle();
        assert!(app.strip.visible);
    }

    #[test]
    fn test_strip_expand_shrink() {
        let mut app = make_app(1);
        app.strip_expand();
        assert_eq!(app.strip.height, 4);
        app.strip_shrink();
        assert_eq!(app.strip.height, 3);
    }

    #[test]
    fn test_strip_shrink_floor() {
        let mut app = make_app(1);
        app.strip.height = 1;
        app.strip_shrink();
        assert_eq!(app.strip.height, 1);
    }

    #[test]
    fn test_filter_mode_set_and_clear() {
        let app = App::new(Workspace::new(vec![]));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.filter_selected, 0);
    }
}
```

- [ ] **Step 4: Run all tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/doobdash/src/app.rs
git commit -m "feat(doobdash): migrate App to Workspace, add Mode::Filter"
```

---

## Task 6: Update `main.rs` — composition root and filter key handler

**Files:**

- Modify: `crates/doobdash/src/main.rs`

- [ ] **Step 1: Replace composition root in `main()`**

The new `main()` wires registry → discovery → parallel load → `Workspace` → `App`.
Replace the entire `main()` function and add `handle_filter()`:

```rust
mod actions;
mod app;
mod data;
mod discovery;
mod ui;
mod workspace;

use anyhow::Result;
use app::{App, Column, Mode, Tab};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use discovery::{FsHandoffDiscovery, HandoffDiscovery, RegistrySource, TomlRegistrySource, source_ref_for_path};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, process::Command, time::Duration};
use workspace::Workspace;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load registry
    let registry_path = TomlRegistrySource::default_path();
    let registry_src = TomlRegistrySource { path: registry_path };
    let config = registry_src.load()?;

    // 2. Discover all HANDOFF files (registry + scan_root)
    let paths: Vec<PathBuf> = if let Some(explicit) = std::env::args().nth(1) {
        vec![PathBuf::from(explicit)]
    } else {
        FsHandoffDiscovery.discover(&config)?
    };

    if paths.is_empty() {
        anyhow::bail!(
            "No HANDOFF.*.yaml files found. Add repos to ~/.ctx/doob/handoffs.toml \
             or run from a repo containing a HANDOFF.*.yaml file."
        );
    }

    // 3. Load each file (sequential; rayon available if needed later)
    let sources: Vec<data::HandoffData> = paths
        .iter()
        .filter_map(|p| {
            let sref = source_ref_for_path(p, &config);
            data::load(p, sref).ok()
        })
        .collect();

    let workspace = Workspace::new(sources);
    let mut app = App::new(workspace);

    // 4. Run TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;

    if app.should_save {
        // Sync each source that was modified
        for source in &app.workspace.sources {
            let status = Command::new("doob")
                .args(["handoff", "sync", "--file", source.handoff_path.to_str().unwrap_or("")])
                .status();
            match status {
                Ok(s) if s.success() => eprintln!("Synced: {}", source.source.name),
                Ok(s) => eprintln!("doob sync exited {}: {}", s, source.source.name),
                Err(e) => eprintln!("doob sync failed ({}): {e}", source.source.name),
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Add `handle_filter()` and wire `p`/`P` in `handle_normal()`**

Add after the existing `handle_search()` function:

```rust
fn handle_filter(app: &mut App, code: KeyCode) {
    let names = app.workspace.project_names();
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.filter_selected = 0;
        }
        KeyCode::Enter => {
            if let Some(name) = names.get(app.filter_selected) {
                app.workspace.active_filter = Some(name.to_string());
            }
            app.mode = Mode::Normal;
            app.filter_selected = 0;
            // Reset column selections after filter change
            app.col_selected = [0; 3];
            app.col_offsets = [0; 3];
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if !names.is_empty() {
                app.filter_selected = (app.filter_selected + 1).min(names.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.filter_selected = app.filter_selected.saturating_sub(1);
        }
        _ => {}
    }
}
```

In `handle_key()`, add `Mode::Filter => handle_filter(app, code),`.

In `handle_normal()`, add these two cases to the main `match code` block (after `'/'`):

```rust
KeyCode::Char('p') => {
    app.mode = Mode::Filter;
    app.filter_selected = 0;
    app.last_key = None;
}
KeyCode::Char('P') => {
    app.workspace.active_filter = None;
    app.col_selected = [0; 3];
    app.col_offsets = [0; 3];
    app.last_key = None;
}
```

- [ ] **Step 3: Fix `selected_id()` usage in `commit_status()` and `handle_input_note()`**

`selected_id()` now returns `Option<String>` (not `Option<&str>`). Calls like
`.map(|s| s.to_string())` should become just `.clone()` or direct use. Update
`commit_status()` in `main.rs`:

```rust
fn commit_status(app: &mut App, status: &str) {
    if let Some(id) = app.selected_id() {
        let path = app.workspace.source_for_item_id(&id)
            .map(|hd| hd.handoff_path.clone());
        if let Some(path) = path {
            let _ = actions::set_status(&path, &id, status);
        }
        // Update in-memory state
        if let Some(idx) = app.selected_item_index() {
            let visible = app.workspace.visible_items();
            if let Some((_, sref)) = visible.get(idx) {
                let sname = sref.name.clone();
                if let Some(source) = app.workspace.sources.iter_mut().find(|hd| hd.source.name == sname) {
                    // find the item in this source and update it
                    let item_id = id.clone();
                    if let Some(item) = source.items.iter_mut().find(|i| i.id == item_id) {
                        item.status = status.to_string();
                    }
                }
            }
        }
        let new_col = match status {
            "done" => Column::Done,
            "parked" | "waiting" => Column::Waiting,
            _ => Column::Active,
        };
        app.active_col = new_col;
    }
    app.mode = Mode::Normal;
    app.status_message = None;
}
```

Update `handle_input_note()` to use `app.selected_id()` directly (it already
returns `Option<String>`, so the `.map(|s| s.to_string())` call can be dropped).

- [ ] **Step 4: Cargo check**

```bash
cd /Users/joe/dev/doob && cargo check -p doobdash 2>&1 | grep "^error" | head -20
```

Expected: no errors.

- [ ] **Step 5: Run tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/doobdash/src/main.rs
git commit -m "feat(doobdash): wire multi-repo composition root and filter key handlers"
```

---

## Task 7: Update `ui.rs` — header badge, card tags, filter picker, per-project stats

**Files:**

- Modify: `crates/doobdash/src/ui.rs`

- [ ] **Step 1: Update `render_header` to show project badge**

Replace the `render_header` function body. The only addition is a filter badge
span after the title:

```rust
fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    // Use the first source's state for branch/build/tests display.
    // When filtered, use the matching source's state.
    let state = app.workspace.sources
        .iter()
        .find(|hd| app.workspace.active_filter.as_deref().map_or(true, |f| hd.source.name == f))
        .map(|hd| &hd.state)
        .unwrap_or(&crate::data::StateData::default());  // need a static default

    let build_color = match state.build.as_str() {
        "ok" | "pass" | "passing" => C_SUCCESS,
        "fail" | "failing" | "error" => C_ERROR,
        _ => C_WARNING,
    };
    let tests_color = match state.tests.as_str() {
        "ok" | "pass" | "passing" => C_SUCCESS,
        "fail" | "failing" | "error" => C_ERROR,
        _ => C_WARNING,
    };

    let active = app.active_count();
    let waiting = app.waiting_count();
    let done = app.done_count();

    let (filter_label, filter_style) = match &app.workspace.active_filter {
        None => ("[all]".to_string(), Style::default().fg(C_MUTED)),
        Some(name) => (format!("[{name}]"), Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
    };

    let line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(filter_label, filter_style),
        Span::styled("  branch: ", Style::default().fg(C_MUTED)),
        Span::styled(&state.branch, Style::default().fg(C_ACCENT)),
        Span::styled("  build: ", Style::default().fg(C_MUTED)),
        Span::styled(&state.build, Style::default().fg(build_color)),
        Span::styled("  tests: ", Style::default().fg(C_MUTED)),
        Span::styled(&state.tests, Style::default().fg(tests_color)),
        Span::styled("  | ", Style::default().fg(C_MUTED)),
        Span::styled(format!("{active}"), Style::default().fg(C_ACTIVE)),
        Span::styled(" active  ", Style::default().fg(C_MUTED)),
        Span::styled(format!("{waiting}"), Style::default().fg(C_WARNING)),
        Span::styled(" waiting  ", Style::default().fg(C_MUTED)),
        Span::styled(format!("{done}"), Style::default().fg(C_SUCCESS)),
        Span::styled(" done ", Style::default().fg(C_MUTED)),
    ]);

    let header = Paragraph::new(line)
        .block(Block::default().borders(Borders::ALL).title(Span::styled(
            " doobdash ",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        )));
    frame.render_widget(header, area);
}
```

Note: `StateData` needs `Default` derived (it already has `#[derive(Default)]`).
Add a `lazy_static` or make a local binding if needed. Simplest approach: add a
`pub fn default_state() -> StateData { StateData::default() }` to `data.rs` and
call that.

- [ ] **Step 2: Update kanban card rendering to show repo tag**

In the function that renders individual kanban items (search for where `ListItem`
is constructed per item), add a repo tag suffix when `active_filter` is `None`.

Find the item rendering loop (around line 220-270 in `ui.rs`). The items are
currently built from `app.data.items`. They now come from
`app.workspace.visible_items()`. The loop change:

```rust
// Before (approximate):
let items_in_col: Vec<ListItem> = col_indices.iter().map(|&i| {
    let item = &app.data.items[i];
    // ... build ListItem
}).collect();

// After:
let visible = app.workspace.visible_items();
let items_in_col: Vec<ListItem> = col_indices.iter().map(|&i| {
    let (item, sref) = &visible[i];
    let show_tag = app.workspace.active_filter.is_none();
    // Build title line
    let mut spans = vec![
        Span::styled(format!(" {} ", item.priority), Style::default().fg(priority_color(&item.priority))),
        Span::styled(&item.title, Style::default().fg(C_BODY)),
    ];
    if show_tag {
        spans.push(Span::styled(format!("  {}", sref.name), Style::default().fg(C_MUTED)));
    }
    ListItem::new(Line::from(spans))
}).collect();
```

Adapt to the actual existing code structure — the logic above shows the pattern,
not a drop-in replacement. Read the existing render loop first and apply minimally.

- [ ] **Step 3: Add `render_filter_picker` and call it from `render()`**

Add a new function at the end of `ui.rs`:

```rust
pub fn render_filter_picker(app: &App, frame: &mut Frame, area: Rect) {
    let names = app.workspace.project_names();
    let items: Vec<ListItem> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.filter_selected {
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(C_BODY)
            };
            ListItem::new(Line::from(Span::styled(format!(" {name} "), style)))
        })
        .collect();

    let height = (names.len() as u16 + 2).min(area.height.saturating_sub(4));
    let width = names.iter().map(|n| n.len() as u16 + 4).max().unwrap_or(20).max(20).min(60);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    let picker_area = Rect::new(x, y, width, height);

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" Filter by project ", Style::default().fg(C_ACCENT)))
            .border_style(Style::default().fg(C_ACCENT)),
    );
    frame.render_widget(ratatui::widgets::Clear, picker_area);
    frame.render_widget(list, picker_area);
}
```

In the `render()` entry point, add a guard at the end — after all other rendering:

```rust
if app.mode == Mode::Filter {
    render_filter_picker(app, frame, area);
}
```

- [ ] **Step 4: Update `render_stats_tab` to include per-project table**

At the end of `render_stats_tab`, add a section for per-project breakdown. After
the existing bar chart, split the remaining area vertically and render a table:

```rust
// Per-project stats table
let stats = app.workspace.per_project_stats();
let rows: Vec<ratatui::widgets::Row> = stats.iter().map(|s| {
    ratatui::widgets::Row::new(vec![
        s.name.clone(),
        s.open.to_string(),
        s.blocked.to_string(),
        s.done.to_string(),
        s.parked.to_string(),
    ])
}).collect();

let table = ratatui::widgets::Table::new(
    rows,
    [
        Constraint::Min(12),
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(7),
    ],
)
.header(ratatui::widgets::Row::new(vec!["Project", "Open", "Blocked", "Done", "Parked"])
    .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)))
.block(Block::default().borders(Borders::ALL).title(" Per-Project "));
frame.render_widget(table, /* stats_area lower half */);
```

Adapt the area splitting in `render_stats_tab` to allocate space for the table.

- [ ] **Step 5: Update footer to mention `p` key**

Find the footer hint line. Add `p filter` or `p proj` to the keybinding hints.

- [ ] **Step 6: Cargo check and fix any compile errors**

```bash
cd /Users/joe/dev/doob && cargo check -p doobdash 2>&1 | grep "^error" | head -30
```

Fix all errors. Common issues: `app.data` references need to become `app.workspace.sources[0]`
or `app.workspace.visible_items()`.

- [ ] **Step 7: Run all tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 8: Build and smoke-test**

```bash
cd /Users/joe/dev/doob && cargo build -p doobdash 2>&1 | tail -10
```

Run manually: `./target/debug/doobdash` from the doob repo root. Verify:

- Header shows `[all]`
- `p` opens the filter picker
- `Enter` selects a project; header updates to `[project-name]`
- `P` clears filter back to `[all]`
- Kanban cards show repo tags when unfiltered

- [ ] **Step 9: Commit**

```bash
git add crates/doobdash/src/ui.rs crates/doobdash/src/data.rs
git commit -m "feat(doobdash): header badge, card repo tags, filter picker, per-project stats"
```

---

## Task 8: Create `~/.ctx/doob/handoffs.toml` for local use

**Files:**

- Create: `~/.ctx/doob/handoffs.toml`

- [ ] **Step 1: Create the registry with the active repo allowlist**

```bash
mkdir -p ~/.ctx/doob
```

Create `~/.ctx/doob/handoffs.toml` with content:

```toml
scan_root = "/Users/joe/dev"

[[repo]]
name = "braid"
path = "/Users/joe/dev/braid"
tags = ["rust"]

[[repo]]
name = "dagu"
path = "/Users/joe/dev/dagu"
tags = ["go", "infra"]

[[repo]]
name = "devkit"
path = "/Users/joe/dev/devkit"
tags = ["rust"]

[[repo]]
name = "devloop"
path = "/Users/joe/dev/devloop"
tags = ["rust", "infra"]

[[repo]]
name = "doob"
path = "/Users/joe/dev/doob"
tags = ["rust", "cli"]

[[repo]]
name = "dotfiles"
path = "/Users/joe/dev/dotfiles"
tags = ["config"]

[[repo]]
name = "harvestrs"
path = "/Users/joe/dev/harvestrs"
tags = ["rust"]

[[repo]]
name = "maestro"
path = "/Users/joe/dev/maestro"
tags = ["rust", "go", "infra"]

[[repo]]
name = "minibox"
path = "/Users/joe/dev/minibox"
tags = ["rust", "infra"]

[[repo]]
name = "obfsck"
path = "/Users/joe/dev/obfsck"
tags = ["rust"]
```

- [ ] **Step 2: Verify it parses**

```bash
cd /Users/joe/dev/doob && cargo run -p doobdash -- --help 2>&1 | head -5
```

Or just run doobdash and verify it finds handoff files from multiple repos.

- [ ] **Step 3: Install and smoke-test**

```bash
cd /Users/joe/dev/doob && cargo install --path crates/doobdash
doobdash
```

Verify the unified board shows items from all repos that have HANDOFF files.

---

## Task 9: Final verification and commit

- [ ] **Step 1: Run full test suite**

```bash
cd /Users/joe/dev/doob && cargo test --all-features 2>&1 | tail -20
```

Expected: all tests pass, no warnings about unused imports.

- [ ] **Step 2: Run clippy**

```bash
cd /Users/joe/dev/doob && cargo clippy -p doobdash -- -D warnings 2>&1 | head -30
```

Fix any warnings.

- [ ] **Step 3: Run CI gate**

```bash
cd /Users/joe/dev/doob && ./ci.sh
```

Expected: pass.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat(doobdash): multi-repo workspace index with filter, registry, and discovery"
```
