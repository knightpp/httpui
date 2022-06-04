use std::time::Duration;

use crossterm::{
    event::{Event, KeyCode},
    Result,
};
use futures::{stream::StreamExt, FutureExt};
use httpfile::HttpRequest;
use tokio::{select, sync::mpsc};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(Some(0));

        StatefulList { state, items }
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
pub struct App {
    channel: (mpsc::Sender<String>, mpsc::Receiver<String>),
    state: AppState,
    data: Data,
}

struct Data {
    items: StatefulList<HttpRequest>,
}

enum AppState {
    ShowingList,
    DoingRequest(HttpRequest),
}

impl AppState {
    fn ui<B: Backend>(&self, f: &mut Frame<B>, data: &mut Data) {
        match self {
            AppState::ShowingList => self.showing_list_ui(f, data),
            AppState::DoingRequest(req) => self.doing_request_ui(f, data, req),
        }
    }

    fn showing_list_ui<B: Backend>(&self, f: &mut Frame<B>, data: &mut Data) {
        // Iterate through all elements in the `items` app and append some debug text to it.
        let items: Vec<ListItem> = data
            .items
            .items
            .iter()
            .map(|req| {
                let main_line = format!("{} {} {}", req.method, req.url, req.version);

                let comment: Spans =
                    Span::styled(req.comment.clone(), Style::default().fg(Color::Yellow)).into();
                let mut lines = vec![comment, Spans::from(main_line)];
                lines.extend(
                    req.headers
                        .iter()
                        .map(|h| format!("{}: {}", h.name, h.value))
                        .map(Spans::from),
                );
                lines.push(Spans::from(req.body.clone()));
                ListItem::new(lines).style(Style::default())
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("HTTP requests"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        // We can now render the item list
        f.render_stateful_widget(items, f.size(), &mut data.items.state);
    }

    fn doing_request_ui<B: Backend>(&self, f: &mut Frame<B>, _data: &Data, req: &HttpRequest) {
        let mut text = vec![Spans::from(Span::raw(""))];
        req.headers
            .iter()
            .map(|header| Spans::from(Span::raw(format!("{}: {}", header.name, header.value))))
            .for_each(|spans| text.push(spans));
        text.push(Spans::from(Span::raw("")));
        text.push(Span::raw(req.body.clone()).into());

        let request_block = Block::default()
            .title(format!("{} {} {}", req.method, req.url, req.version))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        let request_part = Paragraph::new(text).block(request_block);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());
        // f.render_stateful_widget(widget, f.size(), state);
        f.render_widget(request_part, chunks[0]);

        let response_block = Block::default()
            .title("Response")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        f.render_widget(response_block, chunks[1]);
    }
}

impl App {
    pub fn new(items: Vec<HttpRequest>) -> App {
        App {
            channel: mpsc::channel(32),
            data: Data {
                items: StatefulList::with_items(items),
            },
            state: AppState::ShowingList,
        }
    }

    pub fn io_sender(&self) -> mpsc::Sender<String> {
        self.channel.0.clone()
    }

    async fn receive_io(&mut self) -> Option<String> {
        self.channel.1.recv().await
    }

    fn ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.state.ui(f, &mut self.data);
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    terminal.draw(|f| app.ui(f))?;
    let mut event_stream = crossterm::event::EventStream::new();

    // for event in event_stream.next().fuse().await {
    loop {
        let event = event_stream.next().fuse();

        select! {
            event = event => {
                 if let Some(x) = event {
                    handle_terminal_events(&mut app, x?)?;
                } else {
                    break;
                };
            }
            msg = app.receive_io() => {
                if let Some(msg) = msg {
                    // handle_io_events(&mut app, msg)?;
                    panic!("{}", msg);
                } else {
                    break;
                };
            }
        };

        terminal.draw(|f| app.ui(f))?;
    }
    Ok(())
}

fn handle_terminal_events(app: &mut App, event: Event) -> Result<()> {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Left => app.data.items.unselect(),
            KeyCode::Down => app.data.items.next(),
            KeyCode::Up => app.data.items.previous(),
            KeyCode::Enter => {
                let selected = app.data.items.state.selected();
                // app.data.items.items.get(index)
                let selected = selected.and_then(|i| app.data.items.items.get(i));
                if let Some(selected) = selected {
                    app.state = AppState::DoingRequest(selected.clone());

                    let io = app.io_sender();
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                        io.send("".to_string()).await.unwrap();
                    });
                }
            }
            KeyCode::Esc => app.state = AppState::ShowingList,
            _ => {}
        }
    }
    Ok(())
}
