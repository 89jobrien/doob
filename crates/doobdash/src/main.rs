mod actions;
mod app;
mod data;
mod ui;

use anyhow::Result;
use app::{App, Mode};
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
        if let Some(item) = app.data.items.get_mut(app.selected) {
            item.status = status.to_string();
        }
    }
    app.mode = Mode::Normal;
    app.status_message = None;
}
