use rusqlite::{params, Connection, Result};

#[allow(dead_code)]
pub struct User {
    pub username: String,
    pub balance: f64,
}

impl User {
    #[allow(dead_code)]
    pub fn new(username: String, balance: f64) -> Self {
        User { username, balance }
    }

    pub fn create(conn: &Connection, username: &str) -> Result<bool> {
        let user_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)",
            params![username],
            |row| row.get(0),
        )?;

        if user_exists {
            Ok(false)
        } else {
            conn.execute(
                "INSERT INTO users (username, balance) VALUES (?, 0.0)",
                params![username],
            )?;
            Ok(true)
        }
    }

    pub fn get(conn: &Connection, username: &str) -> Result<Option<User>> {
        let mut stmt = conn.prepare("SELECT username, balance FROM users WHERE username = ?")?;
        let mut user_iter = stmt.query_map(params![username], |row| {
            Ok(User {
                username: row.get(0)?,
                balance: row.get(1)?,
            })
        })?;

        user_iter.next().transpose()
    }

    pub fn update_balance(conn: &Connection, username: &str, new_balance: f64) -> Result<()> {
        conn.execute(
            "UPDATE users SET balance = ?1 WHERE username = ?2",
            params![new_balance, username],
        )?;
        Ok(())
    }
}
