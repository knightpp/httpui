use httpfile::HttpRequest;

use crate::widgets::{SpinnerState, StatefulList};

pub struct Model {
    pub state: AppState,
    pub spinner_state: SpinnerState,
    pub items: StatefulList<HttpRequest>,
    pub scroll: Scroll,

    pub resp: Option<String>,
    pub request: Option<HttpRequest>,
}

impl Model {
    pub fn new(items: Vec<HttpRequest>) -> Self {
        Self {
            scroll: Scroll { x: 0, y: 0 },
            request: None,
            resp: None,
            items: StatefulList::with_items(items),
            state: AppState::ShowingList,
            spinner_state: SpinnerState::default(),
        }
    }
}

pub struct Scroll {
    x: u16,
    y: u16,
}

impl Scroll {
    pub fn to_tuple(&self) -> (u16, u16) {
        (self.y, self.x)
    }

    pub fn scroll(&mut self, x: i16, y: i16) {
        self.x = if x.is_negative() {
            self.x.saturating_sub(x.abs() as u16)
        } else {
            self.x.saturating_add(x as u16)
        };
        self.y = if y.is_negative() {
            self.y.saturating_sub(y.abs() as u16)
        } else {
            self.y.saturating_add(y as u16)
        };
    }
}

#[derive(Clone, Copy)]
pub enum AppState {
    ShowingList,
    DoingRequest,
}
