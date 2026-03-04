use crate::models::{Todo, TodoStatus};
use std::collections::BTreeMap;

const CELL_WIDTH: usize = 24;
const BOARD_WIDTH: usize = 70;

pub fn render_board(todos: &[Todo], status_filter: Option<&[TodoStatus]>) -> String {
    // Group: project -> status -> todos
    let mut board: BTreeMap<String, BTreeMap<TodoStatus, Vec<&Todo>>> = BTreeMap::new();

    for todo in todos {
        let project_key = todo
            .project
            .clone()
            .unwrap_or_else(|| "(no project)".to_string());
        let entry = board.entry(project_key).or_default();
        entry.entry(todo.status.clone()).or_default().push(todo);
    }

    if board.is_empty() {
        return "No todos found".to_string();
    }

    let all_statuses = [
        TodoStatus::Pending,
        TodoStatus::InProgress,
        TodoStatus::Completed,
        TodoStatus::Cancelled,
    ];

    let active_statuses: Vec<&TodoStatus> = match status_filter {
        Some(filter) => all_statuses.iter().filter(|s| filter.contains(s)).collect(),
        None => all_statuses.iter().collect(),
    };

    let mut output = String::new();

    for (project, status_map) in &board {
        // Project header
        let header = format!("== project: {} ", project);
        let padding = BOARD_WIDTH.saturating_sub(header.len());
        output.push_str(&format!("{}{}\n", header, "=".repeat(padding)));
        output.push('\n');

        // Determine which columns have data (after filter)
        let columns: Vec<(&TodoStatus, &Vec<&Todo>)> = active_statuses
            .iter()
            .filter_map(|s| status_map.get(s).map(|v| (*s, v)))
            .collect();

        if columns.is_empty() {
            output.push_str("  (no matching todos)\n\n");
            continue;
        }

        // Column headers
        let mut header_row = String::from("  ");
        for (status, todos) in &columns {
            let label = format!("[ {} ({}) ]", status_label(status), todos.len());
            header_row.push_str(&pad_cell(&label));
            header_row.push_str("  ");
        }
        output.push_str(header_row.trim_end());
        output.push('\n');

        // Dividers
        let mut divider_row = String::from("  ");
        for _ in &columns {
            divider_row.push_str(&"\u{2500}".repeat(CELL_WIDTH));
            divider_row.push_str("  ");
        }
        output.push_str(divider_row.trim_end());
        output.push('\n');

        // Rows
        let max_rows = columns.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
        for row in 0..max_rows {
            let mut line = String::from("  ");
            for (_, todos) in &columns {
                if let Some(todo) = todos.get(row) {
                    let short_id = todo
                        .id
                        .as_ref()
                        .map(|t| t.id.to_string())
                        .unwrap_or_else(|| todo.uuid[..6].to_string());
                    let cell = format!("{}  {}", short_id, truncate(&todo.content, 16));
                    line.push_str(&pad_cell(&cell));
                } else {
                    line.push_str(&" ".repeat(CELL_WIDTH));
                }
                line.push_str("  ");
            }
            output.push_str(line.trim_end());
            output.push('\n');
        }

        output.push('\n');
    }

    output
}

fn status_label(status: &TodoStatus) -> &str {
    match status {
        TodoStatus::Pending => "Pending",
        TodoStatus::InProgress => "In Progress",
        TodoStatus::Completed => "Completed",
        TodoStatus::Cancelled => "Cancelled",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn pad_cell(s: &str) -> String {
    if s.len() >= CELL_WIDTH {
        s[..CELL_WIDTH].to_string()
    } else {
        format!("{}{}", s, " ".repeat(CELL_WIDTH - s.len()))
    }
}
