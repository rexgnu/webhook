pub mod app;
pub mod render;

use app::{poll_events, App};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use tokio::sync::mpsc;

use crate::request::CapturedRequest;

pub async fn run_tui(
    listening_address: String,
    mut rx: mpsc::UnboundedReceiver<CapturedRequest>,
) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(listening_address);

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|frame| render::render(frame, &app))?;

        // Handle events
        if let Some(event) = poll_events(&mut rx).await {
            app.handle_input(event);
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
