//! Handoff session models — absorbed from hj.
//!
//! These types represent the YAML-based handoff format, session state,
//! log entries, and survey (handup) reports.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CommitRef
// ---------------------------------------------------------------------------

/// A commit reference in a log entry. Accepts both a bare SHA string and the
/// `{sha, branch}` object form.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub enum CommitRef {
    Sha(String),
    Object {
        sha: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
    },
}

impl CommitRef {
    pub fn sha(&self) -> &str {
        match self {
            CommitRef::Sha(s) => s,
            CommitRef::Object { sha, .. } => sha,
        }
    }
}

fn yaml_value_to_sha(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(format!("{n}")),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

impl<'de> serde::Deserialize<'de> for CommitRef {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let v = serde_yaml::Value::deserialize(de)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;

        match &v {
            serde_yaml::Value::String(s) => return Ok(CommitRef::Sha(s.clone())),
            serde_yaml::Value::Number(n) => return Ok(CommitRef::Sha(format!("{n}"))),
            _ => {}
        }

        if let serde_yaml::Value::Mapping(ref m) = v {
            let sha_key = serde_yaml::Value::String("sha".into());
            let branch_key = serde_yaml::Value::String("branch".into());

            if let Some(sha_val) = m.get(&sha_key) {
                let sha = yaml_value_to_sha(sha_val).ok_or_else(|| {
                    serde::de::Error::custom(format!("commit sha is not a scalar: {sha_val:?}"))
                })?;
                let branch = m
                    .get(&branch_key)
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                return Ok(CommitRef::Object { sha, branch });
            }
        }

        Err(serde::de::Error::custom(format!(
            "expected a SHA string or {{sha, branch}} object, got: {v:?}"
        )))
    }
}

// ---------------------------------------------------------------------------
// LogEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Default)]
pub struct LogEntry {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub commits: Vec<CommitRef>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

impl<'de> serde::Deserialize<'de> for LogEntry {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let map = serde_yaml::Mapping::deserialize(de)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;

        let date = map.get("date").and_then(|v| v.as_str()).map(str::to_string);

        let summary = map
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let commits: Vec<CommitRef> = map
            .get("commits")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| serde_yaml::from_value::<CommitRef>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let mut extra = BTreeMap::new();
        for (k, v) in &map {
            if let Some(key) = k.as_str() {
                if key != "date" && key != "summary" && key != "commits" {
                    extra.insert(key.to_string(), v.clone());
                }
            }
        }

        Ok(LogEntry {
            date,
            summary,
            commits,
            extra,
        })
    }
}

// ---------------------------------------------------------------------------
// Handoff (YAML root)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Handoff {
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
    #[serde(default)]
    pub items: Vec<YamlHandoffItem>,
    #[serde(default)]
    pub log: Vec<LogEntry>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YamlHandoffItem {
    pub id: String,
    #[serde(default)]
    pub doob_uuid: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub completed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<u64>,
    #[serde(default)]
    pub extra: Vec<YamlExtraEntry>,
    #[serde(flatten)]
    pub extra_fields: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YamlExtraEntry {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub field: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub reviewed: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(flatten)]
    pub extra_fields: BTreeMap<String, serde_yaml::Value>,
}

impl Handoff {
    pub fn active_items(&self) -> impl Iterator<Item = &YamlHandoffItem> {
        self.items.iter().filter(|item| item.is_open_or_blocked())
    }
}

impl YamlHandoffItem {
    pub fn is_open_or_blocked(&self) -> bool {
        matches!(self.status.as_deref(), Some("open" | "blocked"))
    }

    pub fn todo_title(&self) -> String {
        let base = self
            .name
            .as_deref()
            .filter(|v| !v.is_empty() && *v != "null")
            .map(titleize_slug)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| self.title.clone());

        if self.status.as_deref() == Some("blocked") {
            format!("{base} [BLOCKED]")
        } else {
            base
        }
    }

    pub fn title_variants(&self) -> Vec<String> {
        let mut variants = Vec::new();
        let title = self.title.clone();
        let blocked_title = format!("{title} [BLOCKED]");
        let todo_title = self.todo_title();
        let blocked_todo_title = if todo_title.ends_with(" [BLOCKED]") {
            todo_title.clone()
        } else {
            format!("{todo_title} [BLOCKED]")
        };

        for value in [title, blocked_title, todo_title, blocked_todo_title] {
            if !value.is_empty() && !variants.iter().any(|existing| existing == &value) {
                variants.push(value);
            }
        }
        variants
    }

    pub fn inferred_priority(&self) -> String {
        self.priority
            .clone()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| infer_priority(&self.title, self.description.as_deref()))
    }
}

// ---------------------------------------------------------------------------
// HandoffState (session metadata — replaces .state.json files)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandoffState {
    #[serde(default)]
    pub updated: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub build: Option<String>,
    #[serde(default)]
    pub tests: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub touched_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_log: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

// ---------------------------------------------------------------------------
// Handup models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct HandupReport {
    pub generated: String,
    pub cwd: String,
    #[serde(default)]
    pub projects: Vec<HandupProject>,
    pub recommendation: HandupRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct HandupProject {
    pub name: String,
    pub path: String,
    pub repo_root: String,
    pub handoff_path: Option<String>,
    pub branch: Option<String>,
    pub build: Option<String>,
    pub tests: Option<String>,
    #[serde(default)]
    pub items: Vec<HandupSummaryItem>,
    #[serde(default)]
    pub todos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct HandupSummaryItem {
    pub id: String,
    pub priority: String,
    pub status: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct HandupRecommendation {
    pub project: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HandupCheckpoint {
    pub project: String,
    pub cwd: String,
    pub generated: String,
    pub recommendation: String,
    pub json_path: String,
}

// ---------------------------------------------------------------------------
// Reconcile types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReconcileMode {
    Sync,
    Audit,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ReconcileReport {
    pub project: String,
    pub captured_count: usize,
    pub created_count: usize,
    pub not_captured: Vec<String>,
    pub orphaned: Vec<String>,
    pub closed_upstream: Vec<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct TodoSnapshot {
    pub active_titles: Vec<String>,
    pub closed_titles: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReconcileCreate {
    pub title: String,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ReconcilePlan {
    pub creates: Vec<ReconcileCreate>,
    pub report: ReconcileReport,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValidationWarning {
    ItemMissingId { index: usize, title: String },
    LogEntryInItems { index: usize, date: String },
    LogEntryMissingSummary { index: usize },
    DuplicateItemId { id: String, indices: Vec<usize> },
}

impl fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationWarning::ItemMissingId { index, title } => {
                write!(f, "items[{index}]: missing id field (title: '{title}')")
            }
            ValidationWarning::LogEntryInItems { index, date } => {
                write!(f, "items[{index}]: looks like a log entry (date: {date})")
            }
            ValidationWarning::LogEntryMissingSummary { index } => {
                write!(f, "log[{index}]: missing summary")
            }
            ValidationWarning::DuplicateItemId { id, indices } => {
                write!(f, "duplicate item id '{id}' at indices {indices:?}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn sanitize_name(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace([' ', '/'], "-")
}

pub fn titleize_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = first.to_uppercase().collect::<String>();
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn infer_priority(title: &str, description: Option<&str>) -> String {
    let title = title.to_ascii_lowercase();
    let description = description.unwrap_or_default().to_ascii_lowercase();
    let combined = format!("{title} {description}");

    if [
        "broken",
        "fails",
        "segfault",
        "panic",
        "security",
        "blocked",
        "urgent",
        "can't deploy",
    ]
    .iter()
    .any(|needle| combined.contains(needle))
    {
        return "P0".to_string();
    }

    if [
        "fix",
        "implement",
        "refactor",
        "wire",
        "small change",
        "known fix",
    ]
    .iter()
    .any(|needle| combined.contains(needle))
    {
        return "P1".to_string();
    }

    "P2".to_string()
}

const PRIORITY_P0: u8 = 5;
const PRIORITY_P1: u8 = 4;
const PRIORITY_P2: u8 = 3;
const PRIORITY_DEFAULT: u8 = 1;

pub fn map_priority(priority: Option<&str>) -> u8 {
    match priority {
        Some("P0") => PRIORITY_P0,
        Some("P1") => PRIORITY_P1,
        Some("P2") => PRIORITY_P2,
        _ => PRIORITY_DEFAULT,
    }
}

pub fn build_reconcile_plan(
    project: &str,
    handoff: &Handoff,
    snapshot: &TodoSnapshot,
    mode: ReconcileMode,
) -> ReconcilePlan {
    let mut captured_count = 0usize;
    let mut created_count = 0usize;
    let mut not_captured = Vec::new();
    let mut closed_upstream = Vec::new();
    let mut creates = Vec::new();
    let mut handoff_titles = std::collections::BTreeSet::new();

    for item in handoff.active_items() {
        for variant in item.title_variants() {
            handoff_titles.insert(variant);
        }

        if contains_any(&snapshot.active_titles, item) {
            captured_count += 1;
            continue;
        }
        if contains_any(&snapshot.closed_titles, item) {
            closed_upstream.push(item.todo_title());
            continue;
        }

        match mode {
            ReconcileMode::Sync => {
                creates.push(ReconcileCreate {
                    title: item.todo_title(),
                    priority: item.priority.clone(),
                });
                captured_count += 1;
                created_count += 1;
            }
            ReconcileMode::Audit => not_captured.push(item.todo_title()),
        }
    }

    let orphaned = snapshot
        .active_titles
        .iter()
        .filter(|title| !handoff_titles.contains(*title))
        .cloned()
        .collect();

    ReconcilePlan {
        creates,
        report: ReconcileReport {
            project: project.to_string(),
            captured_count,
            created_count,
            not_captured,
            orphaned,
            closed_upstream,
        },
    }
}

fn contains_any(existing: &[String], item: &YamlHandoffItem) -> bool {
    item.title_variants()
        .into_iter()
        .any(|variant| existing.iter().any(|title| title == &variant))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn CommitRef_sha_returns_string_for_sha_variant() {
        let r = CommitRef::Sha("abc123".into());
        assert_eq!(r.sha(), "abc123");
    }

    #[test]
    fn CommitRef_sha_returns_sha_from_object_variant() {
        let r = CommitRef::Object {
            sha: "def456".into(),
            branch: Some("main".into()),
        };
        assert_eq!(r.sha(), "def456");
    }

    #[test]
    fn yaml_value_to_sha_converts_string() {
        let v = serde_yaml::Value::String("abc".into());
        assert_eq!(yaml_value_to_sha(&v), Some("abc".into()));
    }

    #[test]
    fn yaml_value_to_sha_returns_none_for_null() {
        let v = serde_yaml::Value::Null;
        assert_eq!(yaml_value_to_sha(&v), None);
    }

    #[test]
    fn YamlHandoffItem_is_open_or_blocked_true_for_open() {
        let item = YamlHandoffItem {
            title: "test".into(),
            status: Some("open".into()),
            ..Default::default()
        };
        assert!(item.is_open_or_blocked());
    }

    #[test]
    fn YamlHandoffItem_is_open_or_blocked_false_for_done() {
        let item = YamlHandoffItem {
            title: "test".into(),
            status: Some("done".into()),
            ..Default::default()
        };
        assert!(!item.is_open_or_blocked());
    }

    #[test]
    fn titleize_slug_capitalizes_each_word() {
        assert_eq!(titleize_slug("hello-world"), "Hello World");
    }

    #[test]
    fn titleize_slug_handles_empty_string() {
        assert_eq!(titleize_slug(""), "");
    }

    #[test]
    fn infer_priority_returns_p0_for_critical() {
        assert_eq!(infer_priority("urgent bug", None), "P0");
    }

    #[test]
    fn infer_priority_returns_p2_for_normal() {
        assert_eq!(infer_priority("add feature", None), "P2");
    }

    #[test]
    fn contains_any_matches_title_variant() {
        let item = YamlHandoffItem {
            title: "my task".into(),
            status: Some("open".into()),
            ..Default::default()
        };
        let existing = vec!["my task".to_string()];
        assert!(contains_any(&existing, &item));
    }

    #[test]
    fn contains_any_no_match() {
        let item = YamlHandoffItem {
            title: "my task".into(),
            status: Some("open".into()),
            ..Default::default()
        };
        let existing = vec!["other task".to_string()];
        assert!(!contains_any(&existing, &item));
    }

    #[test]
    fn Handoff_active_items_filters_open_and_blocked() {
        let handoff = Handoff {
            items: vec![
                YamlHandoffItem {
                    title: "open item".into(),
                    status: Some("open".into()),
                    ..Default::default()
                },
                YamlHandoffItem {
                    title: "done item".into(),
                    status: Some("done".into()),
                    ..Default::default()
                },
                YamlHandoffItem {
                    title: "blocked item".into(),
                    status: Some("blocked".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let active: Vec<_> = handoff.active_items().collect();
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].title, "open item");
        assert_eq!(active[1].title, "blocked item");
    }

    #[test]
    fn YamlHandoffItem_todo_title_uses_name_when_present() {
        let item = YamlHandoffItem {
            title: "raw title".into(),
            name: Some("my-task".into()),
            status: Some("open".into()),
            ..Default::default()
        };
        assert_eq!(item.todo_title(), "My Task");
    }

    #[test]
    fn YamlHandoffItem_title_variants_includes_blocked() {
        let item = YamlHandoffItem {
            title: "my task".into(),
            status: Some("open".into()),
            ..Default::default()
        };
        let variants = item.title_variants();
        assert!(variants.contains(&"my task".to_string()));
        assert!(variants.contains(&"my task [BLOCKED]".to_string()));
    }
}
