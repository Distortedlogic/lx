mod app;
mod event_loop;
mod input;
mod ui;

use std::io;
use std::panic;
use std::sync::Arc;

use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use lx_dx::event::EventBus;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::App;

#[derive(Parser)]
#[command(name = "lx-tui", about = "TUI for lx program observability")]
struct Cli {
    file: String,
}

fn main() {
    let cli = Cli::parse();

    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        default_hook(info);
    }));

    enable_raw_mode().expect("failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .expect("failed to enter alternate screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");

    let bus = Arc::new(EventBus::new());
    let mut app = App::new(cli.file.clone());

    event_loop::start_program(&cli.file, bus.clone());

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(event_loop::run(&mut app, &mut terminal, bus));

    let _ = restore_terminal();
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
