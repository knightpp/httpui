use httpfile::HttpRequest;

use crate::widgets::{SpinnerState, StatefulList};

pub struct Model {
    pub state: AppState,
    pub spinner_state: SpinnerState,
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
            spinner_state: SpinnerState::default(),
        }
    }
}

pub enum AppState {
    ShowingList,
    DoingRequest,
}
