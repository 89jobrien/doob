use crate::app::{App, Column, Mode, Tab};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        BarChart, Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
};

// ---------------------------------------------------------------------------
// Catppuccin Mocha-inspired palette
// ---------------------------------------------------------------------------
const C_ACTIVE: Color = Color::Rgb(137, 180, 250); // Cyan/Blue
const C_SUCCESS: Color = Color::Rgb(166, 227, 161); // Green
const C_WARNING: Color = Color::Rgb(249, 226, 175); // Yellow
const C_ERROR: Color = Color::Rgb(243, 139, 168); // Red
const C_ACCENT: Color = Color::Rgb(203, 166, 247); // Lavender
const C_MUTED: Color = Color::Rgb(88, 91, 112); // Gray
const C_BODY: Color = Color::Rgb(205, 214, 244); // White/Body

fn status_color(status: &str) -> Color {
    match status {
        "done" => C_SUCCESS,
        "blocked" => C_ERROR,
        "parked" | "waiting" => C_WARNING,
        _ => C_BODY,
    }
}

fn priority_color(priority: &str) -> Color {
    match priority {
        "P0" => C_ERROR,
        "P1" => C_WARNING,
        "P2" => C_BODY,
        _ => C_MUTED,
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(3), // tab bar
            Constraint::Min(0),    // content
            Constraint::Length(2), // footer
        ])
        .split(area);

    render_header(app, frame, chunks[0]);
    render_tabs(app, frame, chunks[1]);

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
        Tab::Db => render_db_tab(app, frame, chunks[2]),
    }

    render_footer(app, frame, chunks[3]);
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let state = &app.data.state;

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

    let line = Line::from(vec![
        Span::styled(" branch: ", Style::default().fg(C_MUTED)),
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

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------

fn render_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let tabs = [
        (Tab::Items, "1: Items"),
        (Tab::Log, "2: Log"),
        (Tab::Stats, "3: Stats"),
        (Tab::Help, "4: Help"),
        (Tab::Db, "5: DB"),
    ];

    let spans: Vec<Span> = tabs
        .iter()
        .enumerate()
        .flat_map(|(i, (tab, label))| {
            let is_active = *tab == app.active_tab;
            let style = if is_active {
                Style::default()
                    .fg(C_ACCENT)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(C_MUTED)
            };
            let sep = if i == 0 { " " } else { "  " };
            vec![Span::raw(sep), Span::styled(*label, style)]
        })
        .collect();

    let tab_bar = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(C_MUTED)),
    );
    frame.render_widget(tab_bar, area);
}

// ---------------------------------------------------------------------------
// Items tab — kanban + detail pane
// ---------------------------------------------------------------------------

fn render_items_tab(app: &App, frame: &mut Frame, area: Rect) {
    // Optional search bar at top
    let (search_area, kanban_area) = if matches!(app.mode, Mode::Search) || !app.search_query.is_empty() {
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
        // Cap strip height so kanban always has at least 5 lines
        let max_strip = kanban_area.height.saturating_sub(5);
        let raw_height = app.strip.height + 2; // +2 for top border + padding
        let strip_height = raw_height.min(max_strip);
        if strip_height == 0 {
            (kanban_area, None)
        } else {
            let split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(strip_height)])
                .split(kanban_area);
            (split[0], Some(split[1]))
        }
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
            .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(C_BODY))))
            .collect()
    } else {
        let mut v: Vec<Line> = all_lines[..max_lines - 1]
            .iter()
            .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(C_BODY))))
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

fn render_kanban_col(
    app: &App,
    frame: &mut Frame,
    area: Rect,
    col: Column,
    title: &str,
) {
    let is_focused = app.active_col == col && app.active_tab == Tab::Items;
    let border_style = if is_focused {
        Style::default().fg(C_ACTIVE)
    } else {
        Style::default().fg(C_MUTED)
    };

    let col_idx = col.index();
    let items_idx = app.col_items(col);
    let sel = app.col_selected[col_idx];
    let offset = app.col_offsets[col_idx];

    // Reserve 2 chars for scrollbar
    let inner_width = area.width.saturating_sub(3) as usize;

    let list_items: Vec<ListItem> = items_idx
        .iter()
        .enumerate()
        .map(|(row_i, &data_i)| {
            let item = &app.data.items[data_i];
            let is_sel = is_focused && row_i == sel;

            let pri_span = Span::styled(
                format!("{:<3}", &item.priority),
                Style::default().fg(priority_color(&item.priority)),
            );

            // Truncate title to fit
            let title_max = inner_width.saturating_sub(4);
            let title_str: String = item.title.chars().take(title_max).collect();
            let title_span = Span::styled(
                title_str,
                Style::default().fg(if is_sel { Color::White } else { C_BODY }),
            );

            let line = Line::from(vec![pri_span, Span::raw(" "), title_span]);
            let style = if is_sel {
                Style::default()
                    .bg(Color::Rgb(49, 50, 68))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    if is_focused && !items_idx.is_empty() {
        list_state.select(Some(sel));
    }
    let _ = offset; // scroll offset tracked for future use; ratatui 0.29 drives via select()

    let count = items_idx.len();
    let block_title = format!(" {} ({}) ", title, count);
    let col_color = match col {
        Column::Active => C_ACTIVE,
        Column::Waiting => C_WARNING,
        Column::Done => C_SUCCESS,
    };
    let title_style = Style::default().fg(if is_focused { col_color } else { C_MUTED });

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(block_title, title_style))
                .border_style(border_style),
        )
        .highlight_symbol(if is_focused { "> " } else { "  " });

    // Split area to accommodate scrollbar
    let [list_area, scroll_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .areas(area);

    frame.render_stateful_widget(list, list_area, &mut list_state);

    // Scrollbar
    let total = count;
    if total > 0 {
        let mut scroll_state = ScrollbarState::new(total).position(sel);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(symbols::scrollbar::VERTICAL)
                .style(Style::default().fg(C_MUTED)),
            scroll_area,
            &mut scroll_state,
        );
    }
}

// ---------------------------------------------------------------------------
// Log tab
// ---------------------------------------------------------------------------

fn render_log_tab(app: &App, frame: &mut Frame, area: Rect) {
    let log_items: Vec<ListItem> = app
        .data
        .log
        .iter()
        .map(|e| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}  ", e.date), Style::default().fg(C_MUTED)),
                Span::styled(&e.summary, Style::default().fg(C_BODY)),
            ]))
        })
        .collect();

    let list = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" Log ", Style::default().fg(C_ACCENT)))
            .border_style(Style::default().fg(C_MUTED)),
    );
    frame.render_widget(list, area);
}

// ---------------------------------------------------------------------------
// Stats tab
// ---------------------------------------------------------------------------

fn render_stats_tab(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Min(0)])
        .split(area);

    // Status bar chart
    let statuses = [
        ("open", C_ACTIVE),
        ("blocked", C_ERROR),
        ("parked", C_WARNING),
        ("done", C_SUCCESS),
        ("in-progress", C_ACCENT),
    ];

    let bar_data: Vec<(&str, u64)> = statuses
        .iter()
        .map(|(s, _)| (*s, app.count_by_status(s) as u64))
        .collect();

    let bar_chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(" Status Distribution ", Style::default().fg(C_ACCENT)))
                .border_style(Style::default().fg(C_MUTED)),
        )
        .data(&bar_data)
        .bar_width(9)
        .bar_gap(2)
        .bar_style(Style::default().fg(C_ACTIVE))
        .value_style(Style::default().fg(C_BODY).add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(C_MUTED));
    frame.render_widget(bar_chart, chunks[0]);

    // Priority breakdown
    let priorities = ["P0", "P1", "P2", "P3"];
    let pri_data: Vec<(&str, u64)> = priorities
        .iter()
        .map(|p| {
            (
                *p,
                app.data
                    .items
                    .iter()
                    .filter(|i| i.priority == *p)
                    .count() as u64,
            )
        })
        .collect();

    let pri_chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(" Priority Distribution ", Style::default().fg(C_ACCENT)))
                .border_style(Style::default().fg(C_MUTED)),
        )
        .data(&pri_data)
        .bar_width(9)
        .bar_gap(2)
        .bar_style(Style::default().fg(C_WARNING))
        .value_style(Style::default().fg(C_BODY).add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(C_MUTED));
    frame.render_widget(pri_chart, chunks[1]);
}

// ---------------------------------------------------------------------------
// Help tab
// ---------------------------------------------------------------------------

fn render_help_tab(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(Span::styled("Navigation", Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("")),
        help_row("j / k", "Move down / up within column"),
        help_row("h / l", "Switch column left / right"),
        help_row("gg", "Jump to top of column"),
        help_row("G", "Jump to bottom of column"),
        Line::from(Span::raw("")),
        Line::from(Span::styled("Tabs", Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("")),
        help_row("1", "Items tab (kanban)"),
        help_row("2", "Log tab"),
        help_row("3", "Stats tab"),
        help_row("4 / ?", "Help tab"),
        Line::from(Span::raw("")),
        Line::from(Span::styled("Actions", Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("")),
        help_row("Enter", "Open full detail overlay"),
        help_row("z", "Toggle description strip on/off"),
        help_row("z + j/k", "Shrink/expand strip height (hold z)"),
        help_row("Esc (search)", "Clear search, return to Normal"),
        help_row("q / Esc", "Quit"),
        Line::from(Span::raw("")),
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
    ];

    let para = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(" Help ", Style::default().fg(C_ACCENT)))
                .border_style(Style::default().fg(C_MUTED)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn help_row(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:<20}", key), Style::default().fg(C_ACTIVE)),
        Span::styled(desc.to_owned(), Style::default().fg(C_BODY)),
    ])
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let hint = match (&app.mode, &app.active_tab) {
        (Mode::Search, _) => {
            format!(" SEARCH  type to filter  Esc=clear  /search: {}", app.search_query)
        }
        (Mode::PickStatus, _) => {
            " PICK STATUS  o=open  d=done  p=parked  b=blocked  Esc=cancel".to_string()
        }
        (Mode::InputNote, _) => {
            format!(" INPUT NOTE  Enter=save  Esc=cancel  > {}", app.input_buf)
        }
        (Mode::SpaceLeader, _) => {
            " SPACE  s=status  n=note  w=save  /=search  1-5=tabs  ?=help  Esc=cancel"
                .to_string()
        }
        (Mode::Normal, Tab::Items) => {
            " j/k=nav  h/l=col  gg/G=top/btm  Enter=detail  Space=actions  q=quit".to_string()
        }
        (Mode::Normal, Tab::Log) => " Space=actions  q=quit".to_string(),
        (Mode::Normal, Tab::Stats) => " Space=actions  q=quit".to_string(),
        (Mode::Normal, Tab::Help) => " Space=actions  q=quit".to_string(),
        (Mode::Normal, Tab::Db) => " j/k=nav  Space=actions  q=quit".to_string(),
        (Mode::Overlay, _) => {
            " j/k=scroll  s=status  n=note  Esc=back".to_string()
        }
    };

    // Show status_message override if set
    let display = if let Some(ref msg) = app.status_message {
        msg.as_str().to_owned()
    } else {
        hint
    };

    let footer_style = match app.mode {
        Mode::Search => Style::default().fg(C_ACTIVE),
        Mode::PickStatus => Style::default().fg(C_WARNING),
        Mode::InputNote => Style::default().fg(C_ACCENT),
        Mode::Normal => Style::default().fg(C_MUTED),
        Mode::Overlay => Style::default().fg(C_ACCENT),
        Mode::SpaceLeader => Style::default().fg(C_ACCENT),
    };

    let footer = Paragraph::new(display).style(footer_style).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(C_MUTED)),
    );
    frame.render_widget(footer, area);
}

// ---------------------------------------------------------------------------
// DB tab
// ---------------------------------------------------------------------------

fn render_db_tab(app: &App, frame: &mut Frame, area: Rect) {
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

    if !app.db_loaded {
        let para = Paragraph::new("Loading\u{2026}")
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

    let inner_height = area.height.saturating_sub(2) as usize;
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
            let line = Line::from(vec![
                Span::styled(
                    format!(" {:>4}  ", todo.priority),
                    Style::default().fg(C_MUTED),
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

fn scroll_offset(selected: usize, current_offset: usize, height: usize) -> usize {
    if height == 0 {
        return 0;
    }
    if selected < current_offset {
        selected
    } else if selected >= current_offset + height {
        selected.saturating_sub(height - 1)
    } else {
        current_offset
    }
}
