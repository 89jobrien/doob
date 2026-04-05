use crate::data::HandoffData;
use crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    /// Collecting a new note string for the selected item
    InputNote,
    /// Waiting for a single keypress to pick a status
    PickStatus,
    /// Live search / filter mode
    Search,
    /// Full-screen detail overlay for selected item
    Overlay,
    /// Space was pressed; waiting for action key
    #[allow(dead_code)]
    SpaceLeader,
}

#[derive(Debug)]
pub struct StripState {
    pub visible: bool,
    /// Current height in lines (min 1, default 3)
    pub height: u16,
    /// Instant of last `z` keydown; None when z not held
    pub z_held_since: Option<std::time::Instant>,
}

impl Clone for StripState {
    fn clone(&self) -> Self {
        StripState {
            visible: self.visible,
            height: self.height,
            z_held_since: None, // Instant is not meaningful to clone; reset on clone
        }
    }
}

impl Default for StripState {
    fn default() -> Self {
        StripState {
            visible: true,
            height: 3,
            z_held_since: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Tab {
    Items,
    Log,
    Stats,
    Help,
    #[allow(dead_code)]
    Db,
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

    /// Returns true if item status maps to this column.
    pub fn matches_status(self, status: &str) -> bool {
        match self {
            Column::Active => matches!(status, "open" | "blocked" | "in-progress"),
            Column::Waiting => matches!(status, "parked" | "waiting"),
            Column::Done => status == "done",
        }
    }
}

pub struct App {
    pub data: HandoffData,
    /// Legacy flat index (used by actions/PickStatus to resolve item id)
    pub selected: usize,
    pub mode: Mode,
    pub input_buf: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub should_save: bool,
    // --- new fields ---
    pub active_tab: Tab,
    pub active_col: Column,
    pub search_query: String,
    /// Per-column selection index (into the filtered column items)
    pub col_selected: [usize; 3],
    /// Per-column scroll offset
    pub col_offsets: [usize; 3],
    /// Last key pressed, used for gg detection
    pub last_key: Option<KeyCode>,
    pub strip: StripState,
    /// Scroll offset for the overlay (lines from top)
    pub overlay_scroll: usize,
    // DB tab state
    pub db_todos: Vec<crate::db::DbTodo>,
    pub db_loaded: bool,
    pub db_error: Option<String>,
    pub db_selected: usize,
    #[allow(dead_code)]
    pub db_offset: usize,
    pub db_search: String,
    #[allow(dead_code)]
    pub db_load_requested: bool,
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
            active_tab: Tab::Items,
            active_col: Column::Active,
            search_query: String::new(),
            col_selected: [0; 3],
            col_offsets: [0; 3],
            last_key: None,
            strip: StripState::default(),
            overlay_scroll: 0,
            db_todos: Vec::new(),
            db_loaded: false,
            db_error: None,
            db_selected: 0,
            db_offset: 0,
            db_search: String::new(),
            db_load_requested: false,
        }
    }

    // ---- column-aware navigation ----

    /// Items visible in `col` after applying search filter.
    pub fn col_items(&self, col: Column) -> Vec<usize> {
        let q = self.search_query.to_lowercase();
        self.data
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| col.matches_status(&item.status))
            .filter(|(_, item)| {
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

    /// Index of the currently selected item in `data.items`, if any.
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

    /// Keep the legacy `selected` in sync for actions that still use it.
    fn sync_legacy_selected(&mut self) {
        if let Some(idx) = self.selected_item_index() {
            self.selected = idx;
        }
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected_item_index()
            .and_then(|i| self.data.items.get(i))
            .map(|item| item.id.as_str())
    }

    // ---- stats helpers ----

    pub fn count_by_status(&self, status: &str) -> usize {
        self.data
            .items
            .iter()
            .filter(|i| i.status == status)
            .count()
    }

    pub fn active_count(&self) -> usize {
        self.data
            .items
            .iter()
            .filter(|i| Column::Active.matches_status(&i.status))
            .count()
    }

    pub fn waiting_count(&self) -> usize {
        self.data
            .items
            .iter()
            .filter(|i| Column::Waiting.matches_status(&i.status))
            .count()
    }

    pub fn done_count(&self) -> usize {
        self.data
            .items
            .iter()
            .filter(|i| i.status == "done")
            .count()
    }

    // ---- strip methods ----

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
        self.strip
            .z_held_since
            .map(|t| t.elapsed().as_millis() < 1000)
            .unwrap_or(false)
    }

    pub fn z_press(&mut self) {
        self.strip.z_held_since = Some(std::time::Instant::now());
    }

    pub fn z_release(&mut self) {
        self.strip.z_held_since = None;
    }

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

    #[allow(dead_code)]
    pub fn db_select_next(&mut self) {
        let len = self.db_filtered().len();
        if len > 0 {
            self.db_selected = (self.db_selected + 1).min(len - 1);
        }
    }

    #[allow(dead_code)]
    pub fn db_select_prev(&mut self) {
        self.db_selected = self.db_selected.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{HandoffData, YamlItem};

    fn make_app_with_statuses(statuses: &[&str]) -> App {
        let items = statuses
            .iter()
            .enumerate()
            .map(|(i, s)| YamlItem {
                id: format!("item-{i}"),
                priority: "P1".into(),
                status: s.to_string(),
                title: format!("Item {i}"),
                description: None,
                extra: vec![],
            })
            .collect();
        App::new(HandoffData {
            items,
            ..Default::default()
        })
    }

    fn make_app(n: usize) -> App {
        let statuses: Vec<&str> = (0..n).map(|_| "open").collect();
        make_app_with_statuses(&statuses)
    }

    #[test]
    fn test_select_wraps() {
        let mut app = make_app(3);
        // active col has 3 open items; move to last, next stays clamped
        app.col_selected[0] = 2;
        app.select_next();
        assert_eq!(app.col_selected[0], 2); // clamped at end
    }

    #[test]
    fn test_select_prev_wraps() {
        let mut app = make_app(3);
        app.col_selected[0] = 0;
        app.select_prev();
        assert_eq!(app.col_selected[0], 0); // clamped at 0
    }

    #[test]
    fn test_selected_id() {
        let app = make_app(2);
        assert_eq!(app.selected_id(), Some("item-0"));
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
        app.data.items[0].title = "foo bar".into();
        app.data.items[1].title = "baz qux".into();
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
        assert_eq!(app.strip.height, 1); // floor at 1
    }

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
            id: "t1".into(),
            title: "T".into(),
            status: "open".into(),
            project: "p".into(),
            priority: "P1".into(),
            notes: vec![],
        }];
        app.db_selected = 0;
        app.db_select_next();
        assert_eq!(app.db_selected, 0); // clamped at 0 (only 1 item)
    }
}
