use std::{io::stdout, ops::DerefMut, path::PathBuf};

use app::App;
use clap::Parser;
use crossterm::{
    event::{self, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};

mod app;
mod utils;
mod value;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The file to load.
    file: PathBuf,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    // setup the terminal
    enable_raw_mode()?;
    let mut stderr = std::io::stderr();
    crossterm::execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create the app
    let mut app = App::new(cli.file);

    // main loop
    while !app.done() {
        terminal.draw(|frame| {
            app.draw(frame);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            app.handle_event(Some(event::read()?));
        } else {
            app.handle_event(None);
        }
    }

    // restore terminal
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
