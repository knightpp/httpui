use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::{AppState, Model};
use crate::widgets::Spinner;

pub struct View {}

impl View {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, model: &mut Model) {
        match model.state {
            AppState::ShowingList => self.showing_list_ui(f, model),
            AppState::DoingRequest => self.doing_request_ui(f, model),
        }
    }

    fn showing_list_ui<B: Backend>(&mut self, f: &mut Frame<B>, model: &mut Model) {
        let items: Vec<ListItem> = model
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
        f.render_stateful_widget(items, f.size(), &mut model.items.state);
    }

    fn doing_request_ui<B: Backend>(&mut self, f: &mut Frame<B>, model: &mut Model) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        {
            let mut text = vec![Spans::from("")];
            let req = model.request.clone().unwrap();
            req.headers
                .iter()
                .map(|header| Spans::from(format!("{}: {}", header.name, header.value)))
                .for_each(|spans| text.push(spans));
            text.push(Spans::from(""));

            text.extend(
                req.body
                    .split("\n")
                    .map(|line| Spans::from(vec![Span::raw(line), Span::raw("")])),
            );

            let request_block = Block::default()
                .title(format!("{} {} {}", req.method, req.url, req.version))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White));
            let request_part = Paragraph::new(text).block(request_block);
            f.render_widget(request_part, chunks[0]);
        }
        {
            let response_block = Block::default()
                .title("Response")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White));
            if let Some(body) = &model.resp {
                let response_part = Paragraph::new(body.to_string())
                    .block(response_block)
                    .scroll(model.scroll.to_tuple());
                f.render_widget(response_part, chunks[1]);
            } else {
                let spinner = Spinner::clock()
                    .style(Style::default().fg(Color::Yellow))
                    .block(response_block);
                f.render_stateful_widget(spinner, chunks[1], &mut model.spinner_state);
                // f.render_widget(response_block, chunks[1]);
            }
        }
    }
}
