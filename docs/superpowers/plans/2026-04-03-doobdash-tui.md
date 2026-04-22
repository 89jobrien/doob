---
status: done
---

# doobdash TUI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `doobdash` — a ratatui TUI binary that shows the handoff session-end dashboard
(HANDOFF.yaml items + log + state), with keyboard actions to edit status, append notes, and
write changes back to HANDOFF.yaml.

**Architecture:** Cargo workspace with `doob` as the primary crate and `crates/doobdash` as a
new member. `doobdash` reads HANDOFF.yaml and `.ctx/HANDOFF.state.yaml` directly via serde_yaml
(no DB connection). Mutations shell out to `doob handoff update-status` / `doob handoff
add-extra`. The TUI is a single-screen ratatui app with three fixed panes: state header,
items table, and log list.

**Tech Stack:** Rust 2021, ratatui 0.29, crossterm 0.28, serde_yaml 0.9, tokio 1, anyhow 1

---

## File Map

### New files

| Path                             | Responsibility                                    |
| -------------------------------- | ------------------------------------------------- |
| `Cargo.toml`                     | Convert to workspace; member: `crates/doobdash`   |
| `crates/doobdash/Cargo.toml`     | doobdash crate manifest                           |
| `crates/doobdash/src/main.rs`    | Entry point: parse args, load data, run TUI       |
| `crates/doobdash/src/app.rs`     | `App` state struct, event loop, key dispatch      |
| `crates/doobdash/src/ui.rs`      | `render()` — draws header/items/log panes         |
| `crates/doobdash/src/data.rs`    | `HandoffData` struct, `load()` — reads YAML files |
| `crates/doobdash/src/actions.rs` | `set_status()`, `add_note()` — shell-out helpers  |

### Modified files

| Path         | Change                                                           |
| ------------ | ---------------------------------------------------------------- |
| `Cargo.toml` | Wrap existing package in workspace; add `crates/doobdash` member |

---

## Task 1: Convert to Cargo workspace

**Files:**

- Modify: `Cargo.toml`
- Create: `crates/doobdash/Cargo.toml`

- [ ] **Step 1: Read the current Cargo.toml**

Run: `cat Cargo.toml`

- [ ] **Step 2: Rewrite Cargo.toml as a workspace**

Replace the entire file with:

```toml
[workspace]
members = [".", "crates/doobdash"]
resolver = "2"

[package]
name = "doob"
version = "0.1.0"
edition = "2021"

[lib]
name = "doob"
path = "src/lib.rs"

[[bin]]
name = "doob"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
surrealdb = { version = "2", features = ["kv-surrealkv"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
anyhow = "1"
git2 = "0.19"
uuid = { version = "1", features = ["v4", "serde"] }
dirs-next = "2"
serde_yaml = "0.9"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
serial_test = "3.3.1"

[features]
integration-tests = []
```

- [ ] **Step 3: Create the crate directory and Cargo.toml**

```bash
mkdir -p crates/doobdash/src
```

Create `crates/doobdash/Cargo.toml`:

```toml
[package]
name = "doobdash"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "doobdash"
path = "src/main.rs"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
anyhow = "1"
```

- [ ] **Step 4: Create a stub main.rs so the workspace compiles**

Create `crates/doobdash/src/main.rs`:

```rust
fn main() {
    println!("doobdash");
}
```

- [ ] **Step 5: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: `Finished` with no errors.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/doobdash/Cargo.toml crates/doobdash/src/main.rs
git commit -m "chore: convert doob to cargo workspace; add doobdash stub"
```

---

## Task 2: Data loading (`data.rs`)

**Files:**

- Create: `crates/doobdash/src/data.rs`
- Modify: `crates/doobdash/src/main.rs`

The data layer reads two files:

1. `HANDOFF.*.yaml` — items + log. Path is passed as a CLI argument or auto-detected by
   walking up from CWD looking for `HANDOFF.*.yaml`.
2. `.ctx/HANDOFF.state.yaml` — build/test state. Located relative to the repo root (parent of
   the HANDOFF file).

- [ ] **Step 1: Write the test**

Create `crates/doobdash/src/data.rs` with the test module first:

```rust
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
```

- [ ] **Step 2: Add tempfile dev-dep to doobdash**

In `crates/doobdash/Cargo.toml` add:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p doobdash`
Expected: 2 tests pass (`test_load_items_from_map`, `test_load_missing_state_uses_default`).

- [ ] **Step 4: Commit**

```bash
git add crates/doobdash/
git commit -m "feat(doobdash): data loading from HANDOFF.yaml and state file"
```

---

## Task 3: Shell-out actions (`actions.rs`)

**Files:**

- Create: `crates/doobdash/src/actions.rs`

Actions mutate handoff state by shelling out to the `doob` CLI. This avoids duplicating DB
logic and keeps a clean boundary. Both functions return `Result<()>`.

- [ ] **Step 1: Create `actions.rs`**

```rust
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Shell out: `doob handoff update-status <id> <status> --file <path>`
pub fn set_status(handoff_path: &Path, id: &str, status: &str) -> Result<()> {
    let out = Command::new("doob")
        .args([
            "handoff",
            "update-status",
            id,
            status,
        ])
        .output()
        .with_context(|| "Failed to run doob handoff update-status")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("doob update-status failed: {}", stderr);
    }
    Ok(())
}

/// Shell out: `doob handoff add-extra <id> --type note --note <text>`
pub fn add_note(handoff_path: &Path, id: &str, text: &str) -> Result<()> {
    let out = Command::new("doob")
        .args([
            "handoff",
            "add-extra",
            id,
            "--type",
            "note",
            "--note",
            text,
        ])
        .output()
        .with_context(|| "Failed to run doob handoff add-extra")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("doob add-extra failed: {}", stderr);
    }
    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p doobdash`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add crates/doobdash/src/actions.rs
git commit -m "feat(doobdash): shell-out actions for update-status and add-note"
```

---

## Task 4: App state (`app.rs`)

**Files:**

- Create: `crates/doobdash/src/app.rs`

`App` owns all mutable TUI state: the loaded data, which item is selected, and whether the
user is in a text input mode (for note entry). It also owns the `Mode` enum.

- [ ] **Step 1: Create `app.rs`**

```rust
use crate::data::HandoffData;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    /// Collecting a new note string for the selected item
    InputNote,
    /// Waiting for a single keypress to pick a status
    PickStatus,
}

pub struct App {
    pub data: HandoffData,
    pub selected: usize,
    pub mode: Mode,
    pub input_buf: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub should_save: bool,
}

impl App {
    pub fn new(data: HandoffData) -> Self {
        App {
            data,
            selected: 0,
            mode: Mode::Normal,
            input_buf: String::new(),
            status_message: None,
            should_quit: false,
            should_save: false,
        }
    }

    pub fn select_next(&mut self) {
        let len = self.data.items.len();
        if len > 0 {
            self.selected = (self.selected + 1) % len;
        }
    }

    pub fn select_prev(&mut self) {
        let len = self.data.items.len();
        if len > 0 {
            self.selected = (self.selected + len - 1) % len;
        }
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.data.items.get(self.selected).map(|i| i.id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{HandoffData, YamlItem};

    fn make_app(n: usize) -> App {
        let items = (0..n)
            .map(|i| YamlItem {
                id: format!("item-{i}"),
                priority: "P1".into(),
                status: "open".into(),
                title: format!("Item {i}"),
                description: None,
            })
            .collect();
        App::new(HandoffData { items, ..Default::default() })
    }

    #[test]
    fn test_select_wraps() {
        let mut app = make_app(3);
        app.selected = 2;
        app.select_next();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_select_prev_wraps() {
        let mut app = make_app(3);
        app.selected = 0;
        app.select_prev();
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn test_selected_id() {
        let app = make_app(2);
        assert_eq!(app.selected_id(), Some("item-0"));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p doobdash`
Expected: 5 tests pass.

- [ ] **Step 3: Commit**

```bash
git add crates/doobdash/src/app.rs
git commit -m "feat(doobdash): App state struct with navigation and mode"
```

---

## Task 5: UI rendering (`ui.rs`)

**Files:**

- Create: `crates/doobdash/src/ui.rs`

`render()` takes `&App` and a `&mut Frame` and draws three vertical panes:

- **Header** (3 lines): project, branch, build, test status from `state`
- **Items table** (60% height): columns ID | PRI | STATUS | TITLE, selected row highlighted
- **Log list** (remaining): most recent entries first, date + summary

- [ ] **Step 1: Create `ui.rs`**

```rust
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, TableState},
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(55),
            Constraint::Min(4),
        ])
        .split(area);

    // --- Header ---
    let state = &app.data.state;
    let header_text = format!(
        " branch: {}  build: {}  tests: {}  {}",
        state.branch, state.build, state.tests, state.notes
    );
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(" doobdash "));
    frame.render_widget(header, chunks[0]);

    // --- Items table ---
    let rows: Vec<Row> = app
        .data
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(vec![
                item.id.clone(),
                item.priority.clone(),
                item.status.clone(),
                item.title.clone(),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(4),
        Constraint::Length(10),
        Constraint::Min(20),
    ];
    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["ID", "PRI", "STATUS", "TITLE"])
                .style(Style::default().add_modifier(Modifier::UNDERLINED)),
        )
        .block(Block::default().borders(Borders::ALL).title(" Items "));
    frame.render_widget(table, chunks[1]);

    // --- Log list ---
    let log_items: Vec<ListItem> = app
        .data
        .log
        .iter()
        .map(|e| ListItem::new(format!("{}  {}", e.date, e.summary)))
        .collect();
    let log_list = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title(" Log "));
    frame.render_widget(log_list, chunks[2]);

    // --- Status/mode line overlay at bottom of items pane ---
    if let Some(ref msg) = app.status_message {
        let msg_widget = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Yellow));
        // Render in the last line of items chunk
        let mut msg_area = chunks[1];
        msg_area.y = msg_area.y + msg_area.height.saturating_sub(2);
        msg_area.height = 1;
        msg_area.x += 2;
        msg_area.width = msg_area.width.saturating_sub(4);
        frame.render_widget(msg_widget, msg_area);
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p doobdash`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add crates/doobdash/src/ui.rs
git commit -m "feat(doobdash): ratatui render() with header/items/log panes"
```

---

## Task 6: Main event loop (`main.rs`)

**Files:**

- Modify: `crates/doobdash/src/main.rs`

The event loop:

1. Enter alternate screen + raw mode
2. On each tick: render frame
3. On key event: dispatch to `handle_key()`
4. On `should_quit` or `should_save`: exit (save triggers `doob handoff sync` before quit)

Key bindings:

- `j` / `↓` → select_next
- `k` / `↑` → select_prev
- `s` → enter PickStatus mode (then `o`=open, `d`=done, `p`=parked, `b`=blocked)
- `n` → enter InputNote mode (type text, Enter confirms, Esc cancels)
- `w` → save + quit (runs `doob handoff sync --file <path>`)
- `q` / `Esc` → quit without save

- [ ] **Step 1: Replace `main.rs`**

```rust
mod actions;
mod app;
mod data;
mod ui;

use anyhow::Result;
use app::{App, Mode};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    env,
    io,
    path::PathBuf,
    process::Command,
    time::Duration,
};

fn find_handoff() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    loop {
        for entry in std::fs::read_dir(dir).ok()?.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with("HANDOFF.") && s.ends_with(".yaml") && s != "HANDOFF.state.yaml" {
                return Some(entry.path());
            }
        }
        dir = dir.parent()?;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let handoff_path = match env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => find_handoff().ok_or_else(|| anyhow::anyhow!(
            "No HANDOFF.*.yaml found. Pass path as argument or run from repo root."
        ))?,
    };

    let data = data::load(&handoff_path)?;
    let mut app = App::new(data);

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
        let status = Command::new("doob")
            .args(["handoff", "sync", "--file", handoff_path.to_str().unwrap_or("")])
            .status();
        match status {
            Ok(s) if s.success() => eprintln!("Synced."),
            Ok(s) => eprintln!("doob sync exited with: {}", s),
            Err(e) => eprintln!("doob sync failed: {e}"),
        }
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(app, f))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                handle_key(app, key.code);
            }
        }

        if app.should_quit || app.should_save {
            break;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) {
    match app.mode {
        Mode::Normal => match code {
            KeyCode::Char('j') | KeyCode::Down => app.select_next(),
            KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
            KeyCode::Char('s') => {
                app.mode = Mode::PickStatus;
                app.status_message = Some(
                    "[s]tatus: [o]pen  [d]one  [p]arked  [b]locked  Esc=cancel".to_string(),
                );
            }
            KeyCode::Char('n') => {
                app.mode = Mode::InputNote;
                app.input_buf.clear();
                app.status_message = Some("Note: ".to_string());
            }
            KeyCode::Char('w') => {
                app.should_save = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.should_quit = true;
            }
            _ => {}
        },
        Mode::PickStatus => match code {
            KeyCode::Char('o') => commit_status(app, "open"),
            KeyCode::Char('d') => commit_status(app, "done"),
            KeyCode::Char('p') => commit_status(app, "parked"),
            KeyCode::Char('b') => commit_status(app, "blocked"),
            KeyCode::Esc => {
                app.mode = Mode::Normal;
                app.status_message = None;
            }
            _ => {}
        },
        Mode::InputNote => match code {
            KeyCode::Enter => {
                let text = app.input_buf.trim().to_string();
                if !text.is_empty() {
                    if let Some(id) = app.selected_id().map(|s| s.to_string()) {
                        let _ = actions::add_note(&app.data.handoff_path, &id, &text);
                    }
                }
                app.mode = Mode::Normal;
                app.input_buf.clear();
                app.status_message = None;
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
                app.input_buf.clear();
                app.status_message = None;
            }
            KeyCode::Backspace => {
                app.input_buf.pop();
                app.status_message = Some(format!("Note: {}", app.input_buf));
            }
            KeyCode::Char(c) => {
                app.input_buf.push(c);
                app.status_message = Some(format!("Note: {}", app.input_buf));
            }
            _ => {}
        },
    }
}

fn commit_status(app: &mut App, status: &str) {
    if let Some(id) = app.selected_id().map(|s| s.to_string()) {
        let path = app.data.handoff_path.clone();
        let _ = actions::set_status(&path, &id, status);
        // Optimistically update in-memory
        if let Some(item) = app.data.items.get_mut(app.selected) {
            item.status = status.to_string();
        }
    }
    app.mode = Mode::Normal;
    app.status_message = None;
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build -p doobdash`
Expected: `Finished` with no errors.

- [ ] **Step 3: Smoke test**

From `/Users/joe/dev/doob`:

```bash
cargo run -p doobdash -- HANDOFF.doob.workspace.yaml
```

Expected: TUI launches, shows items and log. Press `q` to exit.

- [ ] **Step 4: Commit**

```bash
git add crates/doobdash/src/main.rs
git commit -m "feat(doobdash): event loop, key bindings, save-and-sync on w"
```

---

## Task 7: Install binary and wire `doob tui` subcommand

**Files:**

- Modify: `src/cli.rs`
- Modify: `src/main.rs`

Add `doob tui` as a thin wrapper that execs `doobdash` with the same arguments.

- [ ] **Step 1: Add Tui variant to Commands in `src/cli.rs`**

Add after the `Watch` variant:

```rust
/// Launch the doobdash TUI dashboard
Tui {
    /// Path to HANDOFF.yaml (auto-detected if omitted)
    #[arg(short = 'f', long)]
    file: Option<String>,
},
```

- [ ] **Step 2: Handle Commands::Tui in `src/main.rs`**

Add the match arm after the `Watch` arm (before the closing `}`):

```rust
Commands::Tui { file } => {
    let mut cmd = std::process::Command::new("doobdash");
    if let Some(f) = file {
        cmd.arg(f);
    }
    let status = cmd.status().context("Failed to launch doobdash — is it installed?")?;
    if !status.success() {
        anyhow::bail!("doobdash exited with: {}", status);
    }
    Ok(())
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --workspace`
Expected: no errors.

- [ ] **Step 4: Install both binaries**

Run: `cargo install --path . && cargo install --path crates/doobdash`
Expected: both install successfully.

- [ ] **Step 5: Integration smoke test**

```bash
doob tui -- HANDOFF.doob.workspace.yaml
```

Or just:

```bash
doobdash HANDOFF.doob.workspace.yaml
```

Expected: TUI loads. `j`/`k` navigates items. `q` quits cleanly.

- [ ] **Step 6: Run full test suite**

Run: `cargo test --workspace`
Expected: all tests pass (no regressions in `doob`).

- [ ] **Step 7: Commit**

```bash
git add src/cli.rs src/main.rs
git commit -m "feat(doob): add tui subcommand that launches doobdash"
```

---

## Task 8: Update HANDOFF.yaml and state

- [ ] **Step 1: Update `.ctx/HANDOFF.state.yaml`**

```yaml
updated: 2026-04-03
branch: main
build: clean
tests: passing
notes: "doobdash TUI implemented"
```

- [ ] **Step 2: Mark doob-2 done in doob**

```bash
doob handoff update-status doob-2 done
```

- [ ] **Step 3: Sync HANDOFF.yaml**

```bash
doob handoff sync --file HANDOFF.doob.workspace.yaml
```

- [ ] **Step 4: Final commit**

```bash
git add HANDOFF.doob.workspace.yaml .ctx/HANDOFF.state.yaml
git commit -m "docs: mark doob-2 done; update handoff state"
```
