use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs)]
/// tui for .http files
pub struct Args {
    /// path to a .http file
    #[argh(positional)]
    pub path: PathBuf,
}

pub fn parse() -> Args {
    argh::from_env()
}
