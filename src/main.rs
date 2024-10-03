mod models;
mod views;
mod controllers;

use std::error::Error;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use controllers::app_controller::AppController;
use views::ui;

fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app controller
    let mut app_controller = AppController::new()?;

    // Main loop
    loop {
        // Clear expired messages
        app_controller.clear_expired_messages();

        // Draw UI
        terminal.draw(|f| ui::draw(f, &app_controller))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if !app_controller.handle_input(key.code)? {
                break;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
