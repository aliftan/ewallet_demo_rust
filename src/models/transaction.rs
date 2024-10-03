use rusqlite::{params, Connection, Result};
use chrono::NaiveDateTime;
use std::collections::HashMap;

pub struct Transaction {
    pub username: String,
    pub transaction_type: String,
    pub amount: f64,
    pub recipient: Option<String>,
    pub sender: Option<String>,
    pub previous_balance: f64,
    pub new_balance: f64,
    pub timestamp: NaiveDateTime,
}

impl Transaction {
    #[allow(dead_code)]  // This suppresses the unused function warning
    pub fn new(
        username: String,
        transaction_type: String,
        amount: f64,
        recipient: Option<String>,
        sender: Option<String>,
        previous_balance: f64,
        new_balance: f64,
    ) -> Self {
        Transaction {
            username,
            transaction_type,
            amount,
            recipient,
            sender,
            previous_balance,
            new_balance,
            timestamp: chrono::Local::now().naive_local(),
        }
    }

    pub fn create(conn: &Connection, transaction: &Transaction) -> Result<()> {
        conn.execute(
            "INSERT INTO transactions (username, transaction_type, amount, recipient, sender, previous_balance, new_balance, timestamp) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                transaction.username,
                transaction.transaction_type,
                transaction.amount,
                transaction.recipient,
                transaction.sender,
                transaction.previous_balance,
                transaction.new_balance,
                transaction.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn get_user_transactions(conn: &Connection, username: &str) -> Result<Vec<HashMap<String, String>>> {
        let mut stmt = conn.prepare(
            "SELECT transaction_type, amount, recipient, sender, previous_balance, new_balance, timestamp 
            FROM transactions 
            WHERE username = ? OR sender = ?
            ORDER BY timestamp DESC"
        )?;
        let transactions = stmt.query_map(params![username, username], |row| {
            let mut transaction = HashMap::new();
            transaction.insert("type".to_string(), row.get(0)?);
            transaction.insert("amount".to_string(), row.get::<_, f64>(1)?.to_string());
            transaction.insert("recipient".to_string(), row.get(2).unwrap_or_default());
            transaction.insert("sender".to_string(), row.get(3).unwrap_or_default());
            transaction.insert("previous_balance".to_string(), row.get::<_, f64>(4)?.to_string());
            transaction.insert("new_balance".to_string(), row.get::<_, f64>(5)?.to_string());
            transaction.insert("timestamp".to_string(), row.get::<_, String>(6)?);
            Ok(transaction)
        })?;
        Ok(transactions.filter_map(Result::ok).collect())
    }
}
