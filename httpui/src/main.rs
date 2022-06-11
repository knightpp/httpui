use std::{
    error::Error,
    fs::File,
    io::{self, BufReader},
    path::Path,
    time::Duration,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mvc::Controller;
use tui::{backend::CrosstermBackend, Terminal};

mod args;
mod mvc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse();

    let https = read_http_file(&args.path)?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    // let app = App::new(https);
    let app = Controller::new(https);
    let tick_rate = Duration::from_millis(200);
    let res = app.run(&mut terminal, tick_rate).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn read_http_file(path: &Path) -> Result<Vec<httpfile::HttpRequest>, Box<dyn Error>> {
    let file = File::open(path)?;
    httpfile::parse(BufReader::new(file)).map_err(Into::into)
}
