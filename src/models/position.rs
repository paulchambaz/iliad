use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Position {
    pub audiobook_hash: String, // primary key
    pub username: String,       // primary key
    pub chapter_index: u32,
    pub chapter_position: u64,
    pub timestamp: NaiveDateTime,
}
