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
use tui::{backend::CrosstermBackend, Terminal};

mod ui;
use ui::*;
mod args;

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse();

    let https = read_http_file(&args.path)?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new(https);
    let res = run_app(&mut terminal, app, tick_rate);

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
