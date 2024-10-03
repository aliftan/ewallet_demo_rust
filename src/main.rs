mod app;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

use crate::app::{App, AppState};

fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;

    // Main loop
    loop {
        // Clear expired messages
        app.clear_expired_messages();

        // Draw UI
        terminal.draw(|f| ui::draw(f, &app))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            match app.current_state {
                AppState::MainMenu => match key.code {
                    KeyCode::Char('1') => app.current_state = AppState::Login,
                    KeyCode::Char('2') => app.current_state = AppState::CreateAccount,
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                AppState::Login | AppState::CreateAccount => match key.code {
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            let success = if app.current_state == AppState::Login {
                                app.login(app.input.clone())?
                            } else {
                                app.create_account(app.input.clone())?
                            };
                            if success {
                                app.input.clear();
                            }
                        }
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.current_state = AppState::MainMenu;
                        app.input.clear();
                    }
                    _ => {}
                },
                AppState::LoggedIn => match key.code {
                    KeyCode::Char('1') => app.current_state = AppState::Deposit,
                    KeyCode::Char('2') => app.current_state = AppState::Withdraw,
                    KeyCode::Char('3') => app.current_state = AppState::Transfer,
                    KeyCode::Char('4') => app.current_state = AppState::ViewTransactions,
                    KeyCode::Char('5') => app.logout(),
                    _ => {}
                },
                AppState::Deposit | AppState::Withdraw => match key.code {
                    KeyCode::Enter => match app.input.trim().parse::<f64>() {
                        Ok(amount) if amount >= 0.0 => {
                            if app.current_state == AppState::Deposit {
                                app.deposit(amount)?;
                            } else {
                                if app.can_withdraw(amount)? {
                                    app.withdraw(amount)?;
                                } else {
                                    app.add_message("Insufficient funds.".to_string());
                                }
                            }
                            app.input.clear();
                            app.current_state = AppState::LoggedIn;
                        }
                        Ok(_) | Err(_) => {
                            app.add_message("Invalid amount entered.".to_string());
                            app.input.clear();
                        }
                    },
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.current_state = AppState::LoggedIn;
                        app.input.clear();
                    }
                    _ => {}
                },
                AppState::Transfer => match key.code {
                    KeyCode::Enter => {
                        if app.transfer_recipient.is_none() {
                            app.transfer_recipient = Some(app.input.clone());
                            app.input.clear();
                        } else {
                            match app.input.trim().parse::<f64>() {
                                Ok(amount) if amount >= 0.0 => {
                                    let recipient = app.transfer_recipient.take().unwrap();
                                    if app.transfer(recipient.clone(), amount)? {
                                        app.input.clear();
                                        app.current_state = AppState::LoggedIn;
                                    }
                                }
                                Ok(_) | Err(_) => {
                                    app.input.clear();
                                    app.add_message("Invalid amount entered.".to_string());
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.current_state = AppState::LoggedIn;
                        app.input.clear();
                        app.transfer_recipient = None;
                    }
                    _ => {}
                },
                AppState::ViewTransactions => {
                    if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                        app.current_state = AppState::LoggedIn;
                    }
                }
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
