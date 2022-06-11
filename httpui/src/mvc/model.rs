use httpfile::HttpRequest;
use tui::widgets::ListState;

pub struct Model {
    pub state: AppState,
    pub items: StatefulList<HttpRequest>,

    pub resp: Option<String>,
    pub request: Option<HttpRequest>,
}

impl Model {
    pub fn new(items: Vec<HttpRequest>) -> Self {
        Self {
            request: None,
            resp: None,
            items: StatefulList::with_items(items),
            state: AppState::ShowingList,
        }
    }
}

pub enum AppState {
    ShowingList,
    DoingRequest,
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(Some(0));

        StatefulList { state, items }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
