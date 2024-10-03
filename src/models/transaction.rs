use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

pub struct Transaction {
    pub id: String,
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
    #[allow(dead_code)] // This suppresses the unused function warning
    pub fn new(
        id: String,
        username: String,
        transaction_type: String,
        amount: f64,
        recipient: Option<String>,
        sender: Option<String>,
        previous_balance: f64,
        new_balance: f64,
    ) -> Self {
        Transaction {
            id,
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
            "INSERT INTO transactions (id, username, transaction_type, amount, recipient, sender, previous_balance, new_balance, timestamp) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                transaction.id,
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

    pub fn get_user_transactions(
        conn: &Connection,
        username: &str,
    ) -> Result<Vec<HashMap<String, String>>> {
        let mut stmt = conn.prepare(
            "SELECT id, transaction_type, amount, recipient, sender, previous_balance, new_balance, timestamp
            FROM transactions
            WHERE username = ? OR sender = ?
            ORDER BY timestamp DESC"
        )?;

        let transactions = stmt.query_map(params![username, username], |row| {
            let transaction_type: String = row.get(1)?;
            let sender: Option<String> = row.get(4)?;

            // Skip this transaction if it's a 'transfer_in' and the sender is the same as the username
            if transaction_type == "transfer_in" && sender.as_deref() == Some(username) {
                return Ok(None);
            }

            let mut transaction = HashMap::new();
            transaction.insert("id".to_string(), row.get::<_, String>(0)?);
            transaction.insert("type".to_string(), transaction_type);
            transaction.insert("amount".to_string(), row.get::<_, f64>(2)?.to_string());
            transaction.insert("recipient".to_string(), row.get(3).unwrap_or_default());
            transaction.insert("sender".to_string(), sender.unwrap_or_default());
            transaction.insert(
                "previous_balance".to_string(),
                row.get::<_, f64>(5)?.to_string(),
            );
            transaction.insert("new_balance".to_string(), row.get::<_, f64>(6)?.to_string());
            transaction.insert("timestamp".to_string(), row.get::<_, String>(7)?);

            Ok(Some(transaction))
        })?;

        Ok(transactions.filter_map(Result::ok).flatten().collect())
    }
}
