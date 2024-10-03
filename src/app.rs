use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct App {
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

impl App {
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
        Ok(App {
            current_state: AppState::MainMenu,
            input: String::new(),
            transfer_recipient: None,
            messages: Vec::new(),
            message_timeout: Duration::from_secs(5), // 5 seconds timeout
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
        let user_exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)",
            params![&username],
            |row| row.get(0),
        )?;

        if user_exists {
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
        let user_exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)",
            params![&username],
            |row| row.get(0),
        )?;

        if user_exists {
            self.add_message(
                "Username already exists. Please choose a different username.".to_string(),
            );
            Ok(false)
        } else {
            self.conn.execute(
                "INSERT INTO users (username, balance) VALUES (?, 0.0)",
                params![&username],
            )?;
            self.current_user = Some(username);
            self.current_state = AppState::LoggedIn;
            self.add_message("Account created successfully.".to_string());
            Ok(true)
        }
    }

    pub fn logout(&mut self) {
        self.current_user = None;
        self.current_state = AppState::MainMenu;
        self.add_message("Logged out successfully.".to_string());
    }

    pub fn deposit(&mut self, amount: f64) -> Result<()> {
        if let Some(username) = &self.current_user {
            let previous_balance: f64 = self.get_balance()?;
            let new_balance = previous_balance + amount;
            self.conn.execute(
                "UPDATE users SET balance = ? WHERE username = ?",
                params![new_balance, username],
            )?;
            self.conn.execute(
                "INSERT INTO transactions (username, transaction_type, amount, previous_balance, new_balance) 
                VALUES (?, 'deposit', ?, ?, ?)",
                params![username, amount, previous_balance, new_balance],
            )?;
            self.add_message(format!("Deposited ${:.2}", amount));
        }
        Ok(())
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<()> {
        if let Some(username) = &self.current_user {
            let previous_balance: f64 = self.get_balance()?;
            let new_balance = previous_balance - amount;
            self.conn.execute(
                "UPDATE users SET balance = ? WHERE username = ?",
                params![new_balance, username],
            )?;
            self.conn.execute(
                "INSERT INTO transactions (username, transaction_type, amount, previous_balance, new_balance) 
                VALUES (?, 'withdraw', ?, ?, ?)",
                params![username, amount, previous_balance, new_balance],
            )?;
            self.add_message(format!("Withdrawn ${:.2}", amount));
        }
        Ok(())
    }

    pub fn can_withdraw(&self, amount: f64) -> Result<bool> {
        if let Some(username) = &self.current_user {
            let balance: f64 = self.conn.query_row(
                "SELECT balance FROM users WHERE username = ?",
                params![username],
                |row| row.get(0),
            )?;
            Ok(balance >= amount)
        } else {
            Ok(false)
        }
    }

    pub fn transfer(&mut self, recipient: String, amount: f64) -> Result<bool> {
        let recipient_exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)",
            params![&recipient],
            |row| row.get(0),
        )?;

        if !recipient_exists {
            self.add_message(format!("Recipient '{}' does not exist.", recipient));
            return Ok(false);
        }

        if !self.can_withdraw(amount)? {
            self.add_message("Insufficient funds for transfer.".to_string());
            return Ok(false);
        }

        let sender = self.current_user.as_ref().unwrap();
        let sender_previous_balance: f64 = self.get_balance()?;
        let sender_new_balance = sender_previous_balance - amount;

        let recipient_previous_balance: f64 = self.conn.query_row(
            "SELECT balance FROM users WHERE username = ?",
            params![&recipient],
            |row| row.get(0),
        )?;
        let recipient_new_balance = recipient_previous_balance + amount;

        self.conn.execute(
            "UPDATE users SET balance = ? WHERE username = ?",
            params![sender_new_balance, sender],
        )?;
        self.conn.execute(
            "UPDATE users SET balance = ? WHERE username = ?",
            params![recipient_new_balance, &recipient],
        )?;
        self.conn.execute(
            "INSERT INTO transactions (username, transaction_type, amount, recipient, sender, previous_balance, new_balance) 
            VALUES (?, 'transfer_out', ?, ?, ?, ?, ?)",
            params![sender, amount, &recipient, sender, sender_previous_balance, sender_new_balance],
        )?;
        self.conn.execute(
            "INSERT INTO transactions (username, transaction_type, amount, recipient, sender, previous_balance, new_balance) 
            VALUES (?, 'transfer_in', ?, ?, ?, ?, ?)",
            params![&recipient, amount, &recipient, sender, recipient_previous_balance, recipient_new_balance],
        )?;
        self.add_message(format!("Transferred ${:.2} to {}", amount, recipient));
        Ok(true)
    }

    pub fn get_balance(&self) -> Result<f64> {
        if let Some(username) = &self.current_user {
            let balance: f64 = self.conn.query_row(
                "SELECT balance FROM users WHERE username = ?",
                params![username],
                |row| row.get(0),
            )?;
            Ok(balance)
        } else {
            Ok(0.0)
        }
    }

    pub fn get_transactions(&self) -> Result<Vec<HashMap<String, String>>> {
        if let Some(username) = &self.current_user {
            let mut stmt = self.conn.prepare(
                "SELECT transaction_type, amount, recipient, sender, previous_balance, new_balance, timestamp 
                FROM transactions 
                WHERE username = ? OR sender = ?
                ORDER BY timestamp DESC 
                LIMIT 10"
            )?;
            let transactions = stmt.query_map(params![username, username], |row| {
                let mut transaction = HashMap::new();
                transaction.insert("type".to_string(), row.get(0)?);
                transaction.insert("amount".to_string(), row.get::<_, f64>(1)?.to_string());
                transaction.insert("recipient".to_string(), row.get(2).unwrap_or_default());
                transaction.insert("sender".to_string(), row.get(3).unwrap_or_default());
                transaction.insert(
                    "previous_balance".to_string(),
                    row.get::<_, f64>(4)?.to_string(),
                );
                transaction.insert("new_balance".to_string(), row.get::<_, f64>(5)?.to_string());
                transaction.insert("timestamp".to_string(), row.get(6)?);
                Ok(transaction)
            })?;
            Ok(transactions.filter_map(Result::ok).collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_current_user(&self) -> Option<&str> {
        self.current_user.as_deref()
    }
}
