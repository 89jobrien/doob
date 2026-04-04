use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
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
        let msg_widget = Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Yellow));
        let mut msg_area = chunks[1];
        msg_area.y = msg_area.y + msg_area.height.saturating_sub(2);
        msg_area.height = 1;
        msg_area.x += 2;
        msg_area.width = msg_area.width.saturating_sub(4);
        frame.render_widget(msg_widget, msg_area);
    }
}
