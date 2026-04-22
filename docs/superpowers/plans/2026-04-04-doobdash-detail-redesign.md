---
status: done
---

# doobdash Detail Pane Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the cramped right-pane detail column with a description strip (bottom of Items
tab, toggleable, resizable) and a full-screen overlay (Enter key).

**Architecture:** Remove the 4th kanban column slot so the 3 columns get equal thirds. Add a
`StripState` struct to `App` tracking visibility + height. Add a `Mode::Overlay` variant with a
scroll offset. All rendering changes are confined to `ui.rs`; all state/input changes to `app.rs`
and `main.rs`.

**Tech Stack:** Rust, ratatui 0.29, crossterm — no new dependencies.

---

## File map

| File                          | Change                                                                                      |
| ----------------------------- | ------------------------------------------------------------------------------------------- |
| `crates/doobdash/src/app.rs`  | Add `StripState`, `Mode::Overlay`, overlay scroll, `z`-held timestamp, strip resize methods |
| `crates/doobdash/src/ui.rs`   | Remove 4th column, add `render_strip`, add `render_overlay`, update `render_items_tab`      |
| `crates/doobdash/src/main.rs` | Handle `Enter`, `Esc` from overlay, `z` tap vs hold + `↑`/`↓`/`j`/`k` in resize mode        |

---

### Task 1: Add `StripState` and `Mode::Overlay` to `app.rs`

**Files:**

- Modify: `crates/doobdash/src/app.rs`

- [ ] **Step 1: Write failing tests for strip state**

Add to the `#[cfg(test)]` block at the bottom of `app.rs`:

```rust
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
    assert_eq!(app.strip.height, 1); // floor at 1
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p doobdash 2>&1 | tail -20
```

Expected: compile error — `strip`, `strip_toggle`, `strip_expand`, `strip_shrink` not defined.

- [ ] **Step 3: Add `StripState`, `Mode::Overlay`, fields and methods**

In `app.rs`, replace the `Mode` enum and add `StripState`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    InputNote,
    PickStatus,
    Search,
    /// Full-screen detail overlay for selected item
    Overlay,
}

#[derive(Debug, Clone)]
pub struct StripState {
    pub visible: bool,
    /// Current height in lines (min 1, default 3)
    pub height: u16,
    /// Unix timestamp (secs) of last `z` keydown; None when z not held
    pub z_held_since: Option<u64>,
}

impl Default for StripState {
    fn default() -> Self {
        StripState { visible: true, height: 3, z_held_since: None }
    }
}
```

Add `strip` and `overlay_scroll` fields to `App`:

```rust
pub struct App {
    pub data: HandoffData,
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
    /// Scroll offset for the overlay (lines from top)
    pub overlay_scroll: usize,
}
```

Update `App::new` to initialise them:

```rust
pub fn new(data: HandoffData) -> Self {
    App {
        data,
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
    }
}
```

Add the strip methods after `done_count`:

```rust
pub fn strip_toggle(&mut self) {
    self.strip.visible = !self.strip.visible;
}

pub fn strip_expand(&mut self) {
    self.strip.height += 1;
}

pub fn strip_shrink(&mut self) {
    if self.strip.height > 1 {
        self.strip.height -= 1;
    }
}

/// Returns true if z is currently considered "held" (pressed within last 1 second).
pub fn z_is_held(&self) -> bool {
    if let Some(t) = self.strip.z_held_since {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(t) < 1
    } else {
        false
    }
}

pub fn z_press(&mut self) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    self.strip.z_held_since = Some(now);
}

pub fn z_release(&mut self) {
    self.strip.z_held_since = None;
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p doobdash 2>&1 | tail -20
```

Expected: all tests pass including the 4 new ones.

- [ ] **Step 5: Commit**

```bash
git add crates/doobdash/src/app.rs
git commit -m "feat(doobdash): add StripState, Mode::Overlay, z-hold resize methods"
```

---

### Task 2: Update `render_items_tab` — remove 4th column, add strip

**Files:**

- Modify: `crates/doobdash/src/ui.rs`

- [ ] **Step 1: Remove the 4th column slot and `render_detail_pane` call**

In `render_items_tab`, replace the layout and render calls:

```rust
fn render_items_tab(app: &App, frame: &mut Frame, area: Rect) {
    // Optional search bar at top
    let (search_area, kanban_area) = if matches!(app.mode, Mode::Search)
        || !app.search_query.is_empty()
    {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);
        (Some(split[0]), split[1])
    } else {
        (None, area)
    };

    if let Some(sa) = search_area {
        let query_display = if matches!(app.mode, Mode::Search) {
            format!("/{}_", app.search_query)
        } else {
            format!("/{}", app.search_query)
        };
        let search_bar = Paragraph::new(query_display).block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(" Search ", Style::default().fg(C_ACTIVE)))
                .border_style(Style::default().fg(C_ACTIVE)),
        );
        frame.render_widget(search_bar, sa);
    }

    // Split vertically: kanban on top, optional strip below
    let (kanban_rect, strip_rect) = if app.strip.visible {
        let strip_height = app.strip.height + 2; // +2 for top border + padding
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(strip_height)])
            .split(kanban_area);
        (split[0], Some(split[1]))
    } else {
        (kanban_area, None)
    };

    // 3 equal kanban columns
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(kanban_rect);

    render_kanban_col(app, frame, cols[0], Column::Active, "Active");
    render_kanban_col(app, frame, cols[1], Column::Waiting, "Waiting");
    render_kanban_col(app, frame, cols[2], Column::Done, "Done");

    if let Some(sr) = strip_rect {
        render_strip(app, frame, sr);
    }
}
```

- [ ] **Step 2: Add `render_strip` function**

Add after `render_items_tab`:

```rust
fn render_strip(app: &App, frame: &mut Frame, area: Rect) {
    let border_style = Style::default().fg(C_MUTED);

    let Some(idx) = app.selected_item_index() else {
        let p = Paragraph::new("")
            .block(Block::default().borders(Borders::TOP).border_style(border_style));
        frame.render_widget(p, area);
        return;
    };

    let item = &app.data.items[idx];
    let desc = item.description.as_deref().unwrap_or("");

    // Collect lines up to strip height, truncate last with … if needed
    let max_lines = app.strip.height as usize;
    let all_lines: Vec<&str> = desc.lines().collect();
    let lines: Vec<Line> = if all_lines.len() <= max_lines {
        all_lines
            .iter()
            .map(|l| Line::from(Span::styled(l.to_owned().to_string(), Style::default().fg(C_BODY))))
            .collect()
    } else {
        let mut v: Vec<Line> = all_lines[..max_lines - 1]
            .iter()
            .map(|l| Line::from(Span::styled(l.to_owned().to_string(), Style::default().fg(C_BODY))))
            .collect();
        let last = all_lines[max_lines - 1];
        let truncated = format!("{}…", last);
        v.push(Line::from(Span::styled(truncated, Style::default().fg(C_BODY))));
        v
    };

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::TOP).border_style(border_style));
    frame.render_widget(para, area);
}
```

- [ ] **Step 3: Delete `render_detail_pane`**

Remove the entire `render_detail_pane` function from `ui.rs` (lines ~304–361 in the current file).
Also remove `render_detail_pane` from the imports/calls — the call was already removed in step 1.

- [ ] **Step 4: Build to verify no compile errors**

```bash
cargo build -p doobdash 2>&1
```

Expected: clean build, no warnings about unused functions.

- [ ] **Step 5: Commit**

```bash
git add crates/doobdash/src/ui.rs
git commit -m "feat(doobdash): replace detail pane with description strip, 3-col kanban"
```

---

### Task 3: Add overlay rendering

**Files:**

- Modify: `crates/doobdash/src/ui.rs`

- [ ] **Step 1: Wire overlay into the top-level `render` function**

In `render`, update the `match app.active_tab` block:

```rust
match app.active_tab {
    Tab::Items => {
        if app.mode == Mode::Overlay {
            render_overlay(app, frame, chunks[2]);
        } else {
            render_items_tab(app, frame, chunks[2]);
        }
    }
    Tab::Log => render_log_tab(app, frame, chunks[2]),
    Tab::Stats => render_stats_tab(app, frame, chunks[2]),
    Tab::Help => render_help_tab(frame, chunks[2]),
}
```

- [ ] **Step 2: Add `render_overlay` function**

Add after `render_strip`:

```rust
fn render_overlay(app: &App, frame: &mut Frame, area: Rect) {
    let Some(idx) = app.selected_item_index() else {
        frame.render_widget(
            Paragraph::new("No item selected")
                .style(Style::default().fg(C_MUTED))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(C_MUTED))),
            area,
        );
        return;
    };

    let item = &app.data.items[idx];

    let mut lines: Vec<Line> = vec![
        // id · priority · status on one line
        Line::from(vec![
            Span::styled("id: ", Style::default().fg(C_MUTED)),
            Span::styled(item.id.clone(), Style::default().fg(C_ACCENT)),
            Span::styled("  priority: ", Style::default().fg(C_MUTED)),
            Span::styled(item.priority.clone(), Style::default().fg(priority_color(&item.priority))),
            Span::styled("  status: ", Style::default().fg(C_MUTED)),
            Span::styled(item.status.clone(), Style::default().fg(status_color(&item.status))),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            item.title.clone(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
    ];

    if let Some(desc) = &item.description {
        lines.push(Line::from(Span::styled(
            "DESCRIPTION",
            Style::default().fg(C_MUTED).add_modifier(Modifier::DIM),
        )));
        for l in desc.lines() {
            lines.push(Line::from(Span::styled(l.to_owned(), Style::default().fg(C_BODY))));
        }
        lines.push(Line::from(Span::raw("")));
    }

    if !item.extra.is_empty() {
        lines.push(Line::from(Span::styled(
            "NOTES",
            Style::default().fg(C_MUTED).add_modifier(Modifier::DIM),
        )));
        for entry in &item.extra {
            lines.push(Line::from(vec![
                Span::styled(format!("[{}] ", entry.date), Style::default().fg(C_MUTED)),
                Span::styled(entry.note.clone(), Style::default().fg(C_BODY)),
            ]));
        }
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_ACCENT)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.overlay_scroll as u16, 0));

    frame.render_widget(para, area);
}
```

- [ ] **Step 3: Update `render_footer` to handle overlay mode**

In `render_footer`, add an `Overlay` arm to the `match (&app.mode, &app.active_tab)`:

```rust
(Mode::Overlay, _) => {
    " j/k=scroll  s=status  n=note  Esc=back".to_string()
}
```

Place it before the `(Mode::Normal, Tab::Items)` arm.

Also add `Overlay` to the `footer_style` match:

```rust
let footer_style = match app.mode {
    Mode::Search => Style::default().fg(C_ACTIVE),
    Mode::PickStatus => Style::default().fg(C_WARNING),
    Mode::InputNote => Style::default().fg(C_ACCENT),
    Mode::Overlay => Style::default().fg(C_ACCENT),
    Mode::Normal => Style::default().fg(C_MUTED),
};
```

- [ ] **Step 4: Build to verify**

```bash
cargo build -p doobdash 2>&1
```

Expected: clean build.

- [ ] **Step 5: Commit**

```bash
git add crates/doobdash/src/ui.rs
git commit -m "feat(doobdash): add full-screen overlay renderer"
```

---

### Task 4: Wire input — Enter/Esc/scroll for overlay, z tap/hold for strip

**Files:**

- Modify: `crates/doobdash/src/main.rs`

- [ ] **Step 1: Add overlay input handler**

Add a new function after `handle_search`:

```rust
fn handle_overlay(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.overlay_scroll = 0;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.overlay_scroll = app.overlay_scroll.saturating_add(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.overlay_scroll = app.overlay_scroll.saturating_sub(1);
        }
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
        _ => {}
    }
}
```

- [ ] **Step 2: Dispatch overlay handler in `handle_key`**

Update `handle_key`:

```rust
fn handle_key(app: &mut App, code: KeyCode) {
    match app.mode {
        Mode::Normal => handle_normal(app, code),
        Mode::PickStatus => handle_pick_status(app, code),
        Mode::InputNote => handle_input_note(app, code),
        Mode::Search => handle_search(app, code),
        Mode::Overlay => handle_overlay(app, code),
    }
}
```

- [ ] **Step 3: Add Enter to open overlay, and z tap/hold logic in `handle_normal`**

In `handle_normal`, add `Enter` handling and replace any existing `z` handling. Add these arms to
the second `match code` block:

```rust
KeyCode::Enter => {
    if app.active_tab == Tab::Items && app.selected_item_index().is_some() {
        app.mode = Mode::Overlay;
        app.overlay_scroll = 0;
    }
    app.last_key = None;
}
KeyCode::Char('z') => {
    if app.z_is_held() {
        // already held — do nothing on repeat keydown, resize handled by up/down
    } else {
        app.z_press();
    }
    app.last_key = Some(code);
}
KeyCode::Up | KeyCode::Char('k') if app.z_is_held() => {
    // z held + up/k → expand strip
    app.strip_expand();
    app.last_key = Some(code);
}
KeyCode::Down | KeyCode::Char('j') if app.z_is_held() => {
    // z held + down/j → shrink strip
    app.strip_shrink();
    app.last_key = Some(code);
}
```

Add a `z` release / tap detection. Because crossterm doesn't fire key-release events by default,
we detect "tap" by checking: if `z` was the last key pressed and the _next_ key is NOT `↑`/`↓`/`j`/`k`,
it was a tap. Update `handle_normal` to check at the top before dispatching:

```rust
fn handle_normal(app: &mut App, code: KeyCode) {
    // If z was held and this keypress is not a resize key, treat the z as a tap (toggle)
    // and release the hold.
    let z_was_held = app.z_is_held();
    let is_resize_key = matches!(
        code,
        KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k')
    );
    if z_was_held && !is_resize_key && code != KeyCode::Char('z') {
        // z pressed then something unrelated — treat as tap
        app.strip_toggle();
        app.z_release();
    } else if !z_was_held && code == KeyCode::Char('z') {
        // fresh z press — record timestamp, wait to see what comes next
        app.z_press();
        app.last_key = Some(code);
        return;
    } else if z_was_held && code == KeyCode::Char('z') {
        // z pressed again while held — ignore
        return;
    }

    // ... rest of existing tab-switching and normal key handling unchanged
```

Add a guard at the top of the normal resize arms (after the z tap/hold block above):

```rust
    if z_was_held && is_resize_key {
        match code {
            KeyCode::Up | KeyCode::Char('k') => app.strip_expand(),
            KeyCode::Down | KeyCode::Char('j') => app.strip_shrink(),
            _ => {}
        }
        app.last_key = Some(code);
        return;
    }
```

- [ ] **Step 4: Release z hold on any non-resize, non-z key**

At the very end of `handle_normal`, before the final `_ =>` arm, add:

```rust
// Release z hold if a non-resize key was processed
if app.z_is_held() {
    app.z_release();
}
```

- [ ] **Step 5: Build and run tests**

```bash
cargo build -p doobdash 2>&1 && cargo test -p doobdash 2>&1 | tail -20
```

Expected: clean build, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/doobdash/src/main.rs
git commit -m "feat(doobdash): wire overlay open/scroll/close and z tap/hold strip resize"
```

---

### Task 5: Update Help tab and smoke test

**Files:**

- Modify: `crates/doobdash/src/ui.rs`

- [ ] **Step 1: Update help text**

In `render_help_tab`, replace the Actions section lines for `s`, `n`, `w`, `/` — and add the new
bindings:

```rust
Line::from(Span::styled("Actions", Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))),
Line::from(Span::raw("")),
help_row("Enter", "Open full detail overlay"),
help_row("Esc (overlay)", "Close overlay, back to kanban"),
help_row("j/k (overlay)", "Scroll overlay"),
help_row("z", "Toggle description strip on/off"),
help_row("z + j/k", "Shrink/expand strip height (hold z)"),
help_row("s", "Set status (o=open d=done p=parked b=blocked)"),
help_row("n", "Add note to selected item"),
help_row("w", "Save + sync to doob"),
help_row("/", "Search / filter items"),
help_row("Esc (search)", "Clear search, return to Normal"),
help_row("q / Esc", "Quit"),
```

- [ ] **Step 2: Manual smoke test**

```bash
cargo install --path crates/doobdash
doobdash
```

Verify:

- Kanban shows 3 equal columns (no 4th pane)
- Description strip visible at bottom with 3 lines of text
- Cursor movement updates strip text
- `z` tap hides strip; kanban expands to fill
- `z` tap again shows strip
- Hold `z`, press `j` — strip shrinks by 1 line
- Hold `z`, press `k` — strip grows by 1 line
- `Enter` opens overlay — shows id/priority/status, title, description, notes
- `j`/`k` scroll overlay
- `s` and `n` work from overlay
- `Esc` closes overlay

- [ ] **Step 3: Final test run**

```bash
cargo test -p doobdash 2>&1
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/doobdash/src/ui.rs
git commit -m "feat(doobdash): update help tab with new keybindings"
```

- [ ] **Step 5: Install and tag**

```bash
cargo install --path crates/doobdash
```

---

## Self-review notes

- `Mode::Overlay` added to all match arms: `handle_key`, `render_footer` footer_style, `render`
  tab dispatch — checked.
- `render_detail_pane` removed from both definition and call site — checked.
- `z_is_held()` uses unix clock seconds matching the `z_press()` timestamp — consistent.
- `strip.height` is `u16`; `Constraint::Length(strip_height)` takes `u16` — types match.
- Overlay scroll uses `app.overlay_scroll as u16` for ratatui's `.scroll()` — correct.
- `entry.r#type` field removed from overlay notes render (spec says date + text only) — checked.
