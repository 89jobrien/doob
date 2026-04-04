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
        App::new(HandoffData {
            items,
            ..Default::default()
        })
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
