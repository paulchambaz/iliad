use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Position {
    pub audiobook_hash: String,
    pub username: String,
    pub file: String,
    pub position: u64,
    pub timestamp: NaiveDateTime,
}
