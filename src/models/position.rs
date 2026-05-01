use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct Position {
    pub audiobook_hash: String, // primary key
    pub username: String,       // primary key
    pub chapter_index: i64,
    pub chapter_position: i64,
    pub timestamp: NaiveDateTime,
}
