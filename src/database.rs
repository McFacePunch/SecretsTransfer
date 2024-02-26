/* use rusqlite::{Connection, Error as SqliteError};
use std::convert::From;

#[derive(Debug)]
pub enum DbError {
    InitError(String),
    SqliteError(SqliteError),
}

impl From<SqliteError> for DbError {
    fn from(err: SqliteError) -> Self {
        DbError::SqliteError(err)
    }
}

pub const DB_PATH: &str = "path_to_your_database.db";

pub fn init_db() -> Result<(), DbError> {
    let conn = Connection::open(DB_PATH).map_err(|e| DbError::InitError(format!("Unable to open the database: {}", e)))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
             username TEXT PRIMARY KEY,
             password TEXT NOT NULL
         )",
        [],
    ).map_err(DbError::from)?;

    Ok(())
}
 */