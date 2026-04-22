---
status: done
---

# Space Leader + DB Tab Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a space-bar leader mode that owns most action keybindings, and a new read-only DB
tab that browses all todos from the SurrealKV store at `~/.ctx/doob/db/`.

**Architecture:** `Mode::SpaceLeader` is a transient mode — space sets it, the next keypress
dispatches an action then returns to Normal (or Esc cancels). The DB tab introduces a
`TodoStore` trait (port) in a new `db.rs` module; `SurrealKvAdapter` implements it. `App` holds
`Option<Box<dyn TodoStore>>` and a `Vec<DbTodo>` cache, loaded lazily on first DB tab visit.

**Tech Stack:** Rust 2021, ratatui 0.29, crossterm 0.28, surrealdb 2 (kv-surrealkv feature),
tokio (already present in doobdash), serde/serde_json.

---

## File Map

| File                          | Change | Responsibility                                              |
| ----------------------------- | ------ | ----------------------------------------------------------- |
| `crates/doobdash/src/app.rs`  | Modify | Add `Mode::SpaceLeader`, `Tab::Db`, `db_todos`, `db_store`  |
| `crates/doobdash/src/main.rs` | Modify | `handle_space_leader`, wire into `handle_key`, lazy DB load |
| `crates/doobdash/src/ui.rs`   | Modify | Space leader footer, DB tab renderer                        |
| `crates/doobdash/src/db.rs`   | Create | `TodoStore` trait, `SurrealKvAdapter`, `DbTodo`             |
| `crates/doobdash/Cargo.toml`  | Modify | Add `surrealdb`, `serde_json` dependencies                  |

---

## Task 1: Add `Mode::SpaceLeader` and `Tab::Db` to app state

**Files:**

- Modify: `crates/doobdash/src/app.rs`

- [ ] **Step 1: Add variants to enums**

In `app.rs`, update `Mode` and `Tab`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    InputNote,
    PickStatus,
    Search,
    Overlay,
    SpaceLeader,  // space was pressed; waiting for action key
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Tab {
    Items,
    Log,
    Stats,
    Help,
    Db,
}
```

- [ ] **Step 2: Add DB fields to `App`**

In `App` struct, add after `overlay_scroll`:

```rust
pub struct App {
    // ... existing fields ...
    pub overlay_scroll: usize,
    // DB tab state
    pub db_todos: Vec<crate::db::DbTodo>,
    pub db_loaded: bool,
    pub db_error: Option<String>,
    pub db_selected: usize,
    pub db_offset: usize,
    pub db_search: String,
}
```

- [ ] **Step 3: Initialize new fields in `App::new`**

```rust
impl App {
    pub fn new(data: HandoffData) -> Self {
        App {
            // ... existing fields ...
            overlay_scroll: 0,
            db_todos: Vec::new(),
            db_loaded: false,
            db_error: None,
            db_selected: 0,
            db_offset: 0,
            db_search: String::new(),
        }
    }
```

- [ ] **Step 4: Add DB navigation helpers to `App`**

```rust
    pub fn db_filtered(&self) -> Vec<&crate::db::DbTodo> {
        let q = self.db_search.to_lowercase();
        self.db_todos
            .iter()
            .filter(|t| {
                q.is_empty()
                    || t.title.to_lowercase().contains(&q)
                    || t.project.to_lowercase().contains(&q)
                    || t.status.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn db_select_next(&mut self) {
        let len = self.db_filtered().len();
        if len > 0 {
            self.db_selected = (self.db_selected + 1).min(len - 1);
        }
    }

    pub fn db_select_prev(&mut self) {
        self.db_selected = self.db_selected.saturating_sub(1);
    }
```

- [ ] **Step 5: Write tests for DB navigation helpers**

Append to `app.rs` `#[cfg(test)]` block:

```rust
    #[test]
    fn test_db_filtered_empty_query_returns_all() {
        let mut app = make_app(1);
        app.db_todos = vec![
            crate::db::DbTodo {
                id: "t1".into(),
                title: "Fix bug".into(),
                status: "open".into(),
                project: "doob".into(),
                priority: "P1".into(),
                notes: vec![],
            },
            crate::db::DbTodo {
                id: "t2".into(),
                title: "Write docs".into(),
                status: "done".into(),
                project: "minibox".into(),
                priority: "P2".into(),
                notes: vec![],
            },
        ];
        assert_eq!(app.db_filtered().len(), 2);
    }

    #[test]
    fn test_db_filtered_by_project() {
        let mut app = make_app(1);
        app.db_todos = vec![
            crate::db::DbTodo {
                id: "t1".into(),
                title: "Fix bug".into(),
                status: "open".into(),
                project: "doob".into(),
                priority: "P1".into(),
                notes: vec![],
            },
            crate::db::DbTodo {
                id: "t2".into(),
                title: "Write docs".into(),
                status: "done".into(),
                project: "minibox".into(),
                priority: "P2".into(),
                notes: vec![],
            },
        ];
        app.db_search = "doob".into();
        assert_eq!(app.db_filtered().len(), 1);
        assert_eq!(app.db_filtered()[0].id, "t1");
    }

    #[test]
    fn test_db_select_next_clamps() {
        let mut app = make_app(1);
        app.db_todos = vec![crate::db::DbTodo {
            id: "t1".into(), title: "T".into(), status: "open".into(),
            project: "p".into(), priority: "P1".into(), notes: vec![],
        }];
        app.db_selected = 0;
        app.db_select_next();
        assert_eq!(app.db_selected, 0); // clamped at 0 (only 1 item)
    }
```

- [ ] **Step 6: Run tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -20
```

Expected: all tests pass (new tests may fail to compile until `db.rs` exists — that's fine, add a
stub in the next task first if needed).

- [ ] **Step 7: Commit**

```bash
cd /Users/joe/dev/doob
git add crates/doobdash/src/app.rs
git commit -m "feat(doobdash): add SpaceLeader mode, Db tab, db nav state"
```

---

## Task 2: Create `db.rs` — `TodoStore` trait and `SurrealKvAdapter`

**Files:**

- Create: `crates/doobdash/src/db.rs`
- Modify: `crates/doobdash/Cargo.toml`
- Modify: `crates/doobdash/src/main.rs` (add `mod db;`)

- [ ] **Step 1: Add surrealdb dependency to doobdash**

In `crates/doobdash/Cargo.toml`, add under `[dependencies]`:

```toml
surrealdb = { version = "2", features = ["kv-surrealkv"] }
serde_json = "1"
dirs-next = "2"
```

- [ ] **Step 2: Create `db.rs` with domain type and trait**

Create `crates/doobdash/src/db.rs`:

```rust
use anyhow::Result;

/// Domain type — doobdash's view of a todo from the DB.
/// Deliberately separate from doob's internal Todo model.
#[derive(Debug, Clone)]
pub struct DbTodo {
    pub id: String,
    pub title: String,
    pub status: String,
    pub project: String,
    pub priority: String,
    pub notes: Vec<String>,
}

/// Port: anything that can supply a list of todos.
pub trait TodoStore: Send + Sync {
    fn list_todos(&self) -> Result<Vec<DbTodo>>;
}

// ---------------------------------------------------------------------------
// SurrealKV adapter
// ---------------------------------------------------------------------------

pub struct SurrealKvAdapter {
    db_path: std::path::PathBuf,
}

impl SurrealKvAdapter {
    /// Use default path: ~/.ctx/doob/db/
    pub fn default_path() -> Result<Self> {
        let home = dirs_next::home_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
        Ok(SurrealKvAdapter {
            db_path: home.join(".ctx/doob/db"),
        })
    }
}

impl TodoStore for SurrealKvAdapter {
    fn list_todos(&self) -> Result<Vec<DbTodo>> {
        // SurrealDB requires a tokio runtime — use block_in_place since doobdash
        // runs inside a tokio::main context.
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                list_todos_async(&self.db_path).await
            })
        })
    }
}

async fn list_todos_async(db_path: &std::path::Path) -> Result<Vec<DbTodo>> {
    use surrealdb::engine::local::SurrealKv;
    use surrealdb::Surreal;

    let db = Surreal::new::<SurrealKv>(db_path).await?;
    db.use_ns("doob").use_db("main").await?;

    // Parameterized queries silently no-op in SurrealDB 2.x (issue #6271).
    // Always use raw SQL strings.
    let mut res = db.query("SELECT * FROM todo").await?;
    let rows: Vec<serde_json::Value> = res.take(0)?;

    let todos = rows
        .into_iter()
        .filter_map(|v| parse_todo(v))
        .collect();

    Ok(todos)
}

fn parse_todo(v: serde_json::Value) -> Option<DbTodo> {
    let id = v.get("id")?.as_str()?.to_string();
    let title = v.get("title")?.as_str().unwrap_or("(untitled)").to_string();
    let status = v
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("open")
        .to_string();
    let project = v
        .get("project")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let priority = v
        .get("priority")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let notes: Vec<String> = v
        .get("notes")
        .and_then(|n| n.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|n| n.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Some(DbTodo { id, title, status, project, priority, notes })
}

// ---------------------------------------------------------------------------
// In-memory test double
// ---------------------------------------------------------------------------

#[cfg(test)]
pub struct InMemoryStore {
    pub todos: Vec<DbTodo>,
}

#[cfg(test)]
impl TodoStore for InMemoryStore {
    fn list_todos(&self) -> Result<Vec<DbTodo>> {
        Ok(self.todos.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_store_returns_todos() {
        let store = InMemoryStore {
            todos: vec![
                DbTodo {
                    id: "todo:abc".into(),
                    title: "Write tests".into(),
                    status: "open".into(),
                    project: "doob".into(),
                    priority: "P1".into(),
                    notes: vec![],
                },
            ],
        };
        let result = store.list_todos().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Write tests");
    }

    #[test]
    fn test_parse_todo_missing_id_returns_none() {
        let v = serde_json::json!({ "title": "No ID todo" });
        assert!(parse_todo(v).is_none());
    }

    #[test]
    fn test_parse_todo_defaults_status_to_open() {
        let v = serde_json::json!({ "id": "todo:1", "title": "A task" });
        let t = parse_todo(v).unwrap();
        assert_eq!(t.status, "open");
    }

    #[test]
    fn test_parse_todo_full() {
        let v = serde_json::json!({
            "id": "todo:abc123",
            "title": "Ship feature",
            "status": "done",
            "project": "doob",
            "priority": "P0",
            "notes": ["First note", "Second note"]
        });
        let t = parse_todo(v).unwrap();
        assert_eq!(t.id, "todo:abc123");
        assert_eq!(t.notes.len(), 2);
        assert_eq!(t.priority, "P0");
    }
}
```

- [ ] **Step 3: Register module in `main.rs`**

At the top of `crates/doobdash/src/main.rs`, add:

```rust
mod db;
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -30
```

Expected: all `db::tests::*` pass.

- [ ] **Step 5: Check clippy**

```bash
cd /Users/joe/dev/doob && cargo clippy -p doobdash 2>&1 | grep -E "error|warning" | head -20
```

Fix any errors before committing.

- [ ] **Step 6: Commit**

```bash
cd /Users/joe/dev/doob
git add crates/doobdash/src/db.rs crates/doobdash/src/main.rs crates/doobdash/Cargo.toml \
    Cargo.lock
git commit -m "feat(doobdash): add TodoStore trait and SurrealKvAdapter in db.rs"
```

---

## Task 3: Wire space leader into the event loop

**Files:**

- Modify: `crates/doobdash/src/main.rs`

- [ ] **Step 1: Route `Mode::SpaceLeader` in `handle_key`**

Update `handle_key`:

```rust
fn handle_key(app: &mut App, code: KeyCode) {
    match app.mode {
        Mode::Normal => handle_normal(app, code),
        Mode::SpaceLeader => handle_space_leader(app, code),
        Mode::PickStatus => handle_pick_status(app, code),
        Mode::InputNote => handle_input_note(app, code),
        Mode::Search => handle_search(app, code),
        Mode::Overlay => handle_overlay(app, code),
    }
}
```

- [ ] **Step 2: In `handle_normal`, replace direct action bindings with space entry**

Replace the body of `handle_normal` after the z-hold logic and tab switching block. The new
Normal-mode match (navigation only) is:

```rust
    // Normal navigation and actions
    match code {
        KeyCode::Char(' ') => {
            app.mode = Mode::SpaceLeader;
            app.last_key = None;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.select_next();
            app.last_key = Some(code);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.select_prev();
            app.last_key = Some(code);
        }
        KeyCode::Char('h') | KeyCode::Left => {
            if app.active_tab == Tab::Items {
                app.col_prev();
            } else if app.active_tab == Tab::Db {
                // no-op in db tab
            }
            app.last_key = Some(code);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if app.active_tab == Tab::Items {
                app.col_next();
            }
            app.last_key = Some(code);
        }
        KeyCode::Char('g') => {
            if app.last_key == Some(KeyCode::Char('g')) {
                app.select_top();
                app.last_key = None;
            } else {
                app.last_key = Some(KeyCode::Char('g'));
            }
        }
        KeyCode::Char('G') => {
            app.select_bottom();
            app.last_key = None;
        }
        KeyCode::Enter => {
            if app.active_tab == Tab::Items && app.selected_item_index().is_some() {
                app.mode = Mode::Overlay;
                app.overlay_scroll = 0;
            }
            app.last_key = None;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {
            app.last_key = Some(code);
        }
    }
```

Also update the tab switching block at the top of `handle_normal` to include `Tab::Db`:

```rust
    match code {
        KeyCode::Char('1') => { app.active_tab = Tab::Items; app.mode = Mode::Normal; app.last_key = None; return; }
        KeyCode::Char('2') => { app.active_tab = Tab::Log;   app.mode = Mode::Normal; app.last_key = None; return; }
        KeyCode::Char('3') => { app.active_tab = Tab::Stats; app.mode = Mode::Normal; app.last_key = None; return; }
        KeyCode::Char('4') | KeyCode::Char('?') => { app.active_tab = Tab::Help; app.mode = Mode::Normal; app.last_key = None; return; }
        _ => {}
    }
```

> Note: Tab switching moves to space leader in Task 3 step 3. Remove this block from Normal after
> adding it to `handle_space_leader`.

- [ ] **Step 3: Add `handle_space_leader` function**

Add after `handle_normal`:

```rust
fn handle_space_leader(app: &mut App, code: KeyCode) {
    match code {
        // Tab switching
        KeyCode::Char('1') => {
            app.active_tab = Tab::Items;
            app.mode = Mode::Normal;
        }
        KeyCode::Char('2') => {
            app.active_tab = Tab::Log;
            app.mode = Mode::Normal;
        }
        KeyCode::Char('3') => {
            app.active_tab = Tab::Stats;
            app.mode = Mode::Normal;
        }
        KeyCode::Char('4') | KeyCode::Char('?') => {
            app.active_tab = Tab::Help;
            app.mode = Mode::Normal;
        }
        KeyCode::Char('5') => {
            app.active_tab = Tab::Db;
            app.mode = Mode::Normal;
            // Trigger lazy load — handled in run_app after handle_key
            app.db_load_requested = true;
        }
        // Actions
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
            app.mode = Mode::Normal;
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Search;
        }
        // Cancel
        KeyCode::Esc | KeyCode::Char(' ') => {
            app.mode = Mode::Normal;
        }
        _ => {
            // Unknown key — cancel leader silently
            app.mode = Mode::Normal;
        }
    }
}
```

- [ ] **Step 4: Add `db_load_requested` field to `App`**

In `app.rs`, add to the `App` struct and `App::new`:

```rust
pub struct App {
    // ... existing fields ...
    pub db_load_requested: bool,
}

impl App {
    pub fn new(data: HandoffData) -> Self {
        App {
            // ... existing fields ...
            db_load_requested: false,
        }
    }
}
```

- [ ] **Step 5: Handle lazy DB load in `run_app`**

In `run_app`, after `handle_key(app, key.code);`, add:

```rust
                handle_key(app, key.code);

                // Lazy DB load when DB tab first activated
                if app.db_load_requested && !app.db_loaded {
                    app.db_load_requested = false;
                    match db::SurrealKvAdapter::default_path() {
                        Ok(adapter) => {
                            match adapter.list_todos() {
                                Ok(todos) => {
                                    app.db_todos = todos;
                                    app.db_loaded = true;
                                    app.db_error = None;
                                }
                                Err(e) => {
                                    app.db_error = Some(format!("DB error: {e}"));
                                    app.db_loaded = true;
                                }
                            }
                        }
                        Err(e) => {
                            app.db_error = Some(format!("DB path error: {e}"));
                            app.db_loaded = true;
                        }
                    }
                }
```

Also add `use crate::db;` to the imports section in `main.rs`.

- [ ] **Step 6: Add j/k navigation for DB tab in `handle_normal`**

In the `j/k` arms, add DB-tab-aware navigation:

```rust
        KeyCode::Char('j') | KeyCode::Down => {
            if app.active_tab == Tab::Db {
                app.db_select_next();
            } else {
                app.select_next();
            }
            app.last_key = Some(code);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.active_tab == Tab::Db {
                app.db_select_prev();
            } else {
                app.select_prev();
            }
            app.last_key = Some(code);
        }
```

- [ ] **Step 7: cargo check**

```bash
cd /Users/joe/dev/doob && cargo check -p doobdash 2>&1 | grep -E "^error" | head -20
```

Fix all errors before continuing.

- [ ] **Step 8: Commit**

```bash
cd /Users/joe/dev/doob
git add crates/doobdash/src/main.rs crates/doobdash/src/app.rs
git commit -m "feat(doobdash): wire SpaceLeader mode and lazy DB load"
```

---

## Task 4: Update UI — space leader footer and DB tab renderer

**Files:**

- Modify: `crates/doobdash/src/ui.rs`

- [ ] **Step 1: Add `Tab::Db` to the tab header bar**

In `render_tabs` (the section that builds the tab spans), add `(Tab::Db, "5: DB")` to the tabs
array:

```rust
    let tabs = [
        (Tab::Items, "1: Items"),
        (Tab::Log,   "2: Log"),
        (Tab::Stats, "3: Stats"),
        (Tab::Help,  "4: Help"),
        (Tab::Db,    "5: DB"),
    ];
```

- [ ] **Step 2: Add `Mode::SpaceLeader` footer hint**

In `render_footer`, add the `SpaceLeader` arm to the hint match:

```rust
        (Mode::SpaceLeader, _) => {
            " SPACE  s=status  n=note  w=save  /=search  1-5=tabs  ?=help  Esc=cancel"
                .to_string()
        }
```

Place it before the `(Mode::Normal, Tab::Items)` arm.

- [ ] **Step 3: Add `Mode::SpaceLeader` footer style**

In the `footer_style` match at the bottom of `render_footer`:

```rust
        Mode::SpaceLeader => Style::default().fg(C_ACCENT),
```

- [ ] **Step 4: Update Normal/Items footer hint**

Remove `s=status  n=note  /=search  w=save` and tab-switching from the Normal/Items hint since
those moved to space:

```rust
        (Mode::Normal, Tab::Items) => {
            " j/k=nav  h/l=col  gg/G=top/btm  Enter=detail  Space=actions  q=quit".to_string()
        }
```

Update other Normal tab hints to reflect space leader:

```rust
        (Mode::Normal, Tab::Log) => " Space=actions  q=quit".to_string(),
        (Mode::Normal, Tab::Stats) => " Space=actions  q=quit".to_string(),
        (Mode::Normal, Tab::Help) => " Space=actions  q=quit".to_string(),
        (Mode::Normal, Tab::Db) => " j/k=nav  Space=actions  q=quit".to_string(),
```

- [ ] **Step 5: Add `Tab::Db` render arm in `render`**

In the main `render` function, add DB tab to the match:

```rust
    match app.active_tab {
        Tab::Items => render_items_tab(app, frame, chunks[2]),
        Tab::Log => render_log_tab(app, frame, chunks[2]),
        Tab::Stats => render_stats_tab(app, frame, chunks[2]),
        Tab::Help => render_help_tab(frame, chunks[2]),
        Tab::Db => render_db_tab(app, frame, chunks[2]),
    }
```

- [ ] **Step 6: Implement `render_db_tab`**

Add the function to `ui.rs`:

```rust
fn render_db_tab(app: &App, frame: &mut Frame, area: Rect) {
    // Show error if DB failed to load
    if let Some(ref err) = app.db_error {
        let para = Paragraph::new(err.as_str())
            .style(Style::default().fg(C_ERROR))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(" DB  error ", Style::default().fg(C_ERROR)))
                    .border_style(Style::default().fg(C_MUTED)),
            );
        frame.render_widget(para, area);
        return;
    }

    // Loading state
    if !app.db_loaded {
        let para = Paragraph::new("Loading…")
            .style(Style::default().fg(C_MUTED))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(" DB ", Style::default().fg(C_ACCENT)))
                    .border_style(Style::default().fg(C_MUTED)),
            );
        frame.render_widget(para, area);
        return;
    }

    let filtered = app.db_filtered();
    let total = filtered.len();
    let title = format!(" DB  {} todos ", total);

    let inner_height = area.height.saturating_sub(2) as usize; // subtract borders
    let offset = scroll_offset(app.db_selected, app.db_offset, inner_height);

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .skip(offset)
        .take(inner_height)
        .map(|(i, todo)| {
            let is_selected = i == app.db_selected;
            let status_color = match todo.status.as_str() {
                "done" => C_SUCCESS,
                "parked" | "waiting" => C_WARNING,
                "blocked" => C_ERROR,
                _ => C_ACTIVE,
            };
            let priority_style = Style::default().fg(C_MUTED);
            let line = Line::from(vec![
                Span::styled(
                    format!(" {:>4}  ", todo.priority),
                    priority_style,
                ),
                Span::styled(
                    format!("{:<10}  ", todo.project),
                    Style::default().fg(C_ACCENT),
                ),
                Span::styled(
                    format!("{:<8}  ", todo.status),
                    Style::default().fg(status_color),
                ),
                Span::styled(todo.title.clone(), Style::default().fg(C_BODY)),
            ]);
            if is_selected {
                ListItem::new(line)
                    .style(Style::default().bg(ratatui::style::Color::DarkGray))
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(title, Style::default().fg(C_ACCENT)))
            .border_style(Style::default().fg(C_MUTED)),
    );

    frame.render_widget(list, area);
}

/// Compute scroll offset so that `selected` stays visible within `height` rows.
fn scroll_offset(selected: usize, current_offset: usize, height: usize) -> usize {
    if selected < current_offset {
        selected
    } else if selected >= current_offset + height {
        selected.saturating_sub(height - 1)
    } else {
        current_offset
    }
}
```

> Note: `scroll_offset` is a pure helper — add it near the top of `ui.rs` or beside
> `render_db_tab`. Check that `List`, `ListItem`, `Line`, `Span` are already imported (they
> are used elsewhere in `ui.rs`).

- [ ] **Step 7: Update help tab with new keybindings**

In `render_help_tab`, update the Actions section to reflect space leader:

```rust
        Line::from(Span::styled("Space Leader", Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("")),
        help_row("Space", "Enter space leader (action mode)"),
        help_row("Space s", "Set status (o=open d=done p=parked b=blocked)"),
        help_row("Space n", "Add note to selected item"),
        help_row("Space w", "Save + sync to doob"),
        help_row("Space /", "Search / filter items"),
        help_row("Space 1-5", "Switch tabs"),
        help_row("Space ?", "Help tab"),
        help_row("Esc (leader)", "Cancel space leader"),
```

Remove the old direct-key action entries (`s`, `n`, `w`, `/`) from the Actions section.

- [ ] **Step 8: cargo check**

```bash
cd /Users/joe/dev/doob && cargo check -p doobdash 2>&1 | grep "^error" | head -20
```

Fix all errors.

- [ ] **Step 9: Run all tests**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -20
```

Expected: all pass.

- [ ] **Step 10: Commit**

```bash
cd /Users/joe/dev/doob
git add crates/doobdash/src/ui.rs
git commit -m "feat(doobdash): add DB tab renderer and space leader footer"
```

---

## Task 5: Final integration check and install

**Files:** none new

- [ ] **Step 1: Full clippy pass**

```bash
cd /Users/joe/dev/doob && cargo clippy -p doobdash -- -D warnings 2>&1 | head -40
```

Fix all warnings.

- [ ] **Step 2: Full test suite**

```bash
cd /Users/joe/dev/doob && cargo test -p doobdash 2>&1 | tail -20
```

Expected: all pass.

- [ ] **Step 3: Install and smoke test**

```bash
cargo install --path /Users/joe/dev/doob/crates/doobdash
doobdash
```

Verify:

- Tab bar shows `1: Items  2: Log  3: Stats  4: Help  5: DB`
- Pressing `s` in Normal mode does nothing (no status picker)
- Pressing `Space` shows the space leader footer hint
- Pressing `Space s` opens status picker
- Pressing `Space 5` switches to DB tab
- DB tab shows "Loading…" then populates (or shows error if DB absent)
- `j/k` navigates DB list
- `Space /` enters search from DB tab

- [ ] **Step 4: Update CLAUDE.md keybindings line**

In `/Users/joe/dev/doob/CLAUDE.md`, update the doobdash keybindings entry:

```markdown
- Keybindings: `j/k` nav col · `h/l` switch col · `Enter` overlay · `Space` leader (actions/tabs)
  Space leader: `s`=status · `n`=note · `w`=save · `/`=search · `1-5`=tabs · `?`=help · `Esc`=cancel
  `z` toggle strip · `z+j/k` resize strip · `q` quit · `5: DB` tab browses SurrealKV todos
```

- [ ] **Step 5: Final commit**

```bash
cd /Users/joe/dev/doob
git add /Users/joe/dev/doob/CLAUDE.md
git commit -m "docs: update doobdash keybindings for space leader and DB tab"
```
