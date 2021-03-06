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
use httpui::mvc::Controller;
use tui::{backend::CrosstermBackend, Terminal};

mod args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse();
    let https = read_http_file(&args.path)?
        .into_iter()
        .map(|mut req| {
            req.body = serde_json::from_str::<serde_json::Value>(&req.body)
                .and_then(|obj| serde_json::to_string_pretty(&obj))
                .unwrap_or(req.body);
            req
        })
        .collect();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
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
