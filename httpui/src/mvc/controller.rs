use std::time::Duration;

use anyhow::{anyhow, Result};
use crossterm::event::{Event, KeyCode};
use futures::{stream::StreamExt, FutureExt};
use httpfile::HttpRequest;
use tokio::time;
use tokio::{select, sync::mpsc};
use tui::backend::Backend;
use tui::Terminal;

use super::model::Model;
use super::view::View;
use super::AppState;

enum AppAction {
    Exit,
    Continue,
}

pub struct Controller {
    model: Model,
    view: View,

    client: reqwest::Client,
    channel: (mpsc::Sender<String>, mpsc::Receiver<String>),
}

impl Controller {
    pub fn new(items: Vec<HttpRequest>) -> Controller {
        Controller {
            model: Model::new(items),
            view: View::new(),

            client: reqwest::Client::new(),
            channel: mpsc::channel(32),
        }
    }

    pub async fn run<B: Backend>(
        mut self,
        terminal: &mut Terminal<B>,
        tick: Duration,
    ) -> Result<()> {
        terminal.draw(|f| self.view.render(f, &mut self.model))?;

        let mut event_stream = crossterm::event::EventStream::new();
        let mut interval = time::interval(tick);

        'outer: loop {
            let event = event_stream.next().fuse();
            let tick = interval.tick();

            select! {
                event = event => {
                    if let Some(e ) = event{
                        if let AppAction::Exit = self.on_event(e?).await?{
                            break 'outer;
                        }
                    }else{
                        break 'outer;
                    }
                }
                _ = tick =>{
                    self.on_tick().await?;
                }
                msg = self.receive_io() => {
                    if let Some(msg) = msg {
                        self.on_io(msg).await?;
                    }else{
                        return Err(anyhow!("No message received"));
                    }
                }
            }

            terminal.draw(|f| self.view.render(f, &mut self.model))?;
        }
        Ok(())
    }
}

impl Controller {
    pub fn io_sender(&self) -> mpsc::Sender<String> {
        self.channel.0.clone()
    }

    async fn receive_io(&mut self) -> Option<String> {
        self.channel.1.recv().await
    }

    async fn on_event(&mut self, event: Event) -> Result<AppAction> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => return Ok(AppAction::Exit),
                KeyCode::Left => self.model.items.unselect(),
                KeyCode::Down => self.model.items.next(),
                KeyCode::Up => self.model.items.previous(),
                KeyCode::Enter => {
                    let selected = self.model.items.state.selected();
                    // app.data.items.items.get(index)
                    let selected = selected.and_then(|i| self.model.items.items.get(i));
                    if let Some(selected) = selected {
                        self.model.state = AppState::DoingRequest;
                        self.model.request = Some(selected.clone());

                        let io = self.io_sender();
                        // TOOD: remove unwrap
                        let req = selected.to_reqwest(&self.client).unwrap();
                        let client = self.client.clone();

                        tokio::spawn(async move {
                            let resp = client.execute(req).await.unwrap();
                            let body = resp.text().await.unwrap();
                            io.send(body).await.unwrap();
                        });
                    }
                }
                KeyCode::Esc => self.model.state = AppState::ShowingList,
                _ => {}
            }
        }
        Ok(AppAction::Continue)
    }

    async fn on_tick(&mut self) -> Result<()> {
        Ok(())
    }

    async fn on_io(&mut self, msg: String) -> Result<()> {
        self.model.resp = Some(msg);
        Ok(())
    }
}
