mod actions;
mod app;
mod data;
mod ui;

use anyhow::Result;
use app::{App, Column, Mode, Tab};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io, path::PathBuf, process::Command, time::Duration};

fn find_handoff() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    loop {
        for entry in std::fs::read_dir(dir).ok()?.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with("HANDOFF.")
                && s.ends_with(".yaml")
                && s != "HANDOFF.state.yaml"
            {
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
        None => find_handoff().ok_or_else(|| {
            anyhow::anyhow!(
                "No HANDOFF.*.yaml found. Pass path as argument or run from repo root."
            )
        })?,
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
            .args([
                "handoff",
                "sync",
                "--file",
                handoff_path.to_str().unwrap_or(""),
            ])
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
        Mode::Normal => handle_normal(app, code),
        Mode::PickStatus => handle_pick_status(app, code),
        Mode::InputNote => handle_input_note(app, code),
        Mode::Search => handle_search(app, code),
        Mode::Overlay => handle_overlay(app, code),
    }
}

fn handle_normal(app: &mut App, code: KeyCode) {
    // If z was held and this keypress is not a resize key, treat the z as a tap (toggle)
    // and release the hold.
    let z_was_held = app.z_is_held();
    let is_resize_key = matches!(
        code,
        KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k')
    );
    if z_was_held && !is_resize_key && code != KeyCode::Char('z') {
        // z pressed then something unrelated — treat as tap (toggle strip)
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

    // z held + resize key → adjust strip height
    if z_was_held && is_resize_key {
        match code {
            KeyCode::Up | KeyCode::Char('k') => app.strip_expand(),
            KeyCode::Down | KeyCode::Char('j') => app.strip_shrink(),
            _ => {}
        }
        app.last_key = Some(code);
        return;
    }

    // Tab switching — close overlay if open, then switch
    match code {
        KeyCode::Char('1') => {
            app.mode = Mode::Normal;
            app.active_tab = Tab::Items;
            app.last_key = None;
            return;
        }
        KeyCode::Char('2') => {
            app.mode = Mode::Normal;
            app.active_tab = Tab::Log;
            app.last_key = None;
            return;
        }
        KeyCode::Char('3') => {
            app.mode = Mode::Normal;
            app.active_tab = Tab::Stats;
            app.last_key = None;
            return;
        }
        KeyCode::Char('4') | KeyCode::Char('?') => {
            app.mode = Mode::Normal;
            app.active_tab = Tab::Help;
            app.last_key = None;
            return;
        }
        _ => {}
    }

    // Normal navigation and actions
    match code {
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
        KeyCode::Char('s') => {
            app.mode = Mode::PickStatus;
            app.status_message = Some(
                "[s]tatus: [o]pen  [d]one  [p]arked  [b]locked  Esc=cancel".to_string(),
            );
            app.last_key = None;
        }
        KeyCode::Char('n') => {
            app.mode = Mode::InputNote;
            app.input_buf.clear();
            app.status_message = Some("Note: ".to_string());
            app.last_key = None;
        }
        KeyCode::Char('w') => {
            app.should_save = true;
            app.last_key = None;
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Search;
            app.last_key = None;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {
            app.last_key = Some(code);
        }
    }

    // Release z hold if a non-resize key was processed
    if app.z_is_held() {
        app.z_release();
    }
}

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

fn handle_pick_status(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('o') => commit_status(app, "open"),
        KeyCode::Char('d') => commit_status(app, "done"),
        KeyCode::Char('p') => commit_status(app, "parked"),
        KeyCode::Char('b') => commit_status(app, "blocked"),
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.status_message = None;
        }
        _ => {}
    }
}

fn handle_input_note(app: &mut App, code: KeyCode) {
    match code {
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
    }
}

fn handle_search(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.search_query.clear();
        }
        KeyCode::Enter => {
            app.mode = Mode::Normal;
        }
        KeyCode::Backspace => {
            app.search_query.pop();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
        }
        _ => {}
    }
}

fn commit_status(app: &mut App, status: &str) {
    if let Some(id) = app.selected_id().map(|s| s.to_string()) {
        let path = app.data.handoff_path.clone();
        let _ = actions::set_status(&path, &id, status);
        // Update in-memory state
        if let Some(idx) = app.selected_item_index() {
            if let Some(item) = app.data.items.get_mut(idx) {
                item.status = status.to_string();
            }
        }
        // After status change the item may move columns — re-sync
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
