use crate::models::{transaction::Transaction, user::User};
use crossterm::event::KeyCode;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct AppController {
    pub current_state: AppState,
    pub input: String,
    pub transfer_recipient: Option<String>,
    pub messages: Vec<(String, Instant)>,
    message_timeout: Duration,
    conn: Connection,
    current_user: Option<String>,
}

#[derive(PartialEq)]
pub enum AppState {
    MainMenu,
    Login,
    CreateAccount,
    LoggedIn,
    Deposit,
    Withdraw,
    Transfer,
    ViewTransactions,
}

impl AppController {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("ewallet.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                balance REAL NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                transaction_type TEXT NOT NULL,
                amount REAL NOT NULL,
                recipient TEXT,
                sender TEXT,
                previous_balance REAL NOT NULL,
                new_balance REAL NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(AppController {
            current_state: AppState::MainMenu,
            input: String::new(),
            transfer_recipient: None,
            messages: Vec::new(),
            message_timeout: Duration::from_secs(5),
            conn,
            current_user: None,
        })
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push((message, Instant::now()));
    }

    pub fn clear_expired_messages(&mut self) {
        let now = Instant::now();
        self.messages
            .retain(|(_, timestamp)| now.duration_since(*timestamp) < self.message_timeout);
    }

    pub fn login(&mut self, username: String) -> Result<bool> {
        if let Some(_user) = User::get(&self.conn, &username)? {
            self.current_user = Some(username);
            self.current_state = AppState::LoggedIn;
            self.add_message("Login successful.".to_string());
            Ok(true)
        } else {
            self.add_message("User does not exist. Please try again.".to_string());
            Ok(false)
        }
    }    

    pub fn create_account(&mut self, username: String) -> Result<bool> {
        if User::create(&self.conn, &username)? {
            self.current_user = Some(username);
            self.current_state = AppState::LoggedIn;
            self.add_message("Account created successfully.".to_string());
            Ok(true)
        } else {
            self.add_message(
                "Username already exists. Please choose a different username.".to_string(),
            );
            Ok(false)
        }
    }

    pub fn logout(&mut self) {
        self.current_user = None;
        self.current_state = AppState::MainMenu;
        self.add_message("Logged out successfully.".to_string());
    }

    pub fn deposit(&mut self, amount: f64) -> Result<()> {
        if let Some(username) = &self.current_user {
            let previous_balance = self.get_balance()?;
            let new_balance = previous_balance + amount;
            User::update_balance(&self.conn, username, new_balance)?;

            let transaction = Transaction {
                username: username.clone(),
                transaction_type: "deposit".to_string(),
                amount,
                recipient: None,
                sender: None,
                previous_balance,
                new_balance,
                timestamp: chrono::Local::now().naive_local(),
            };
            Transaction::create(&self.conn, &transaction)?;

            self.add_message(format!("Deposited ${:.2}", amount));
        }
        Ok(())
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<()> {
        if let Some(username) = &self.current_user {
            let previous_balance = self.get_balance()?;
            let new_balance = previous_balance - amount;
            User::update_balance(&self.conn, username, new_balance)?;

            let transaction = Transaction {
                username: username.clone(),
                transaction_type: "withdraw".to_string(),
                amount,
                recipient: None,
                sender: None,
                previous_balance,
                new_balance,
                timestamp: chrono::Local::now().naive_local(),
            };
            Transaction::create(&self.conn, &transaction)?;

            self.add_message(format!("Withdrawn ${:.2}", amount));
        }
        Ok(())
    }

    pub fn can_withdraw(&self, amount: f64) -> Result<bool> {
        if let Some(username) = &self.current_user {
            if let Some(user) = User::get(&self.conn, username)? {
                Ok(user.balance >= amount)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub fn transfer(&mut self, recipient: String, amount: f64) -> Result<bool> {
        if let (Some(sender_username), Some(recipient_user)) =
            (&self.current_user, User::get(&self.conn, &recipient)?)
        {
            let sender_previous_balance = self.get_balance()?;
            
            // Check if sender has sufficient balance
            if sender_previous_balance < amount {
                self.add_message(format!(
                    "Transfer failed. Insufficient funds. Your balance: ${:.2}",
                    sender_previous_balance
                ));
                return Ok(false);
            }
    
            let sender_new_balance = sender_previous_balance - amount;
            let recipient_previous_balance = recipient_user.balance;
            let recipient_new_balance = recipient_previous_balance + amount;
    
            User::update_balance(&self.conn, sender_username, sender_new_balance)?;
            User::update_balance(&self.conn, &recipient, recipient_new_balance)?;
    
            let sender_transaction = Transaction {
                username: sender_username.clone(),
                transaction_type: "transfer_out".to_string(),
                amount,
                recipient: Some(recipient.clone()),
                sender: None,
                previous_balance: sender_previous_balance,
                new_balance: sender_new_balance,
                timestamp: chrono::Local::now().naive_local(),
            };
            Transaction::create(&self.conn, &sender_transaction)?;
    
            let recipient_transaction = Transaction {
                username: recipient.clone(),
                transaction_type: "transfer_in".to_string(),
                amount,
                recipient: None,
                sender: Some(sender_username.clone()),
                previous_balance: recipient_previous_balance,
                new_balance: recipient_new_balance,
                timestamp: chrono::Local::now().naive_local(),
            };
            Transaction::create(&self.conn, &recipient_transaction)?;
    
            self.add_message(format!("Transferred ${:.2} to {}", amount, recipient));
            Ok(true)
        } else {
            self.add_message(format!(
                "Transfer failed. Recipient '{}' not found.",
                recipient
            ));
            Ok(false)
        }
    }    

    pub fn get_balance(&self) -> Result<f64> {
        if let Some(username) = &self.current_user {
            if let Some(user) = User::get(&self.conn, username)? {
                Ok(user.balance)
            } else {
                Ok(0.0)
            }
        } else {
            Ok(0.0)
        }
    }

    pub fn get_transactions(&self) -> Result<Vec<HashMap<String, String>>> {
        if let Some(username) = &self.current_user {
            Transaction::get_user_transactions(&self.conn, username)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_current_user(&self) -> Option<&str> {
        self.current_user.as_deref()
    }

    pub fn handle_input(&mut self, key: KeyCode) -> Result<bool> {
        match self.current_state {
            AppState::MainMenu => match key {
                KeyCode::Char('1') => self.current_state = AppState::Login,
                KeyCode::Char('2') => self.current_state = AppState::CreateAccount,
                KeyCode::Char('q') => return Ok(false),
                _ => {}
            },
            AppState::Login | AppState::CreateAccount => match key {
                KeyCode::Enter => {
                    if !self.input.is_empty() {
                        let success = if self.current_state == AppState::Login {
                            self.login(self.input.clone())?
                        } else {
                            self.create_account(self.input.clone())?
                        };
                        if success {
                            self.input.clear();
                        }
                    }
                }
                KeyCode::Char(c) => self.input.push(c),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Esc => {
                    self.current_state = AppState::MainMenu;
                    self.input.clear();
                }
                _ => {}
            },
            AppState::LoggedIn => match key {
                KeyCode::Char('1') => self.current_state = AppState::Deposit,
                KeyCode::Char('2') => self.current_state = AppState::Withdraw,
                KeyCode::Char('3') => self.current_state = AppState::Transfer,
                KeyCode::Char('4') => self.current_state = AppState::ViewTransactions,
                KeyCode::Char('5') => self.logout(),
                _ => {}
            },
            AppState::Deposit | AppState::Withdraw => match key {
                KeyCode::Enter => {
                    if let Ok(amount) = self.input.trim().parse::<f64>() {
                        if amount >= 0.0 {
                            if self.current_state == AppState::Deposit {
                                self.deposit(amount)?;
                            } else if self.can_withdraw(amount)? {
                                self.withdraw(amount)?;
                            } else {
                                self.add_message("Insufficient funds.".to_string());
                            }
                            self.input.clear();
                            self.current_state = AppState::LoggedIn;
                        } else {
                            self.add_message(
                                "Invalid amount. Please enter a positive number.".to_string(),
                            );
                        }
                    } else {
                        self.add_message(
                            "Invalid amount. Please enter a valid number.".to_string(),
                        );
                    }
                }
                KeyCode::Char(c) => self.input.push(c),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Esc => {
                    self.current_state = AppState::LoggedIn;
                    self.input.clear();
                }
                _ => {}
            },
            AppState::Transfer => match key {
                KeyCode::Enter => {
                    if self.transfer_recipient.is_none() {
                        self.transfer_recipient = Some(self.input.clone());
                        self.input.clear();
                    } else {
                        if let Ok(amount) = self.input.trim().parse::<f64>() {
                            if amount >= 0.0 {
                                let recipient = self.transfer_recipient.take().unwrap();
                                self.transfer(recipient, amount)?;
                                self.input.clear();
                                self.current_state = AppState::LoggedIn;
                            } else {
                                self.add_message(
                                    "Invalid amount. Please enter a positive number.".to_string(),
                                );
                            }
                        } else {
                            self.add_message(
                                "Invalid amount. Please enter a valid number.".to_string(),
                            );
                        }
                    }
                }
                KeyCode::Char(c) => self.input.push(c),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Esc => {
                    self.current_state = AppState::LoggedIn;
                    self.input.clear();
                    self.transfer_recipient = None;
                }
                _ => {}
            },
            AppState::ViewTransactions => {
                if key == KeyCode::Esc || key == KeyCode::Enter {
                    self.current_state = AppState::LoggedIn;
                }
            }
        }
        Ok(true)
    }
}
