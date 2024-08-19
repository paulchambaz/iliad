use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Position {
    pub id: i64,
    pub user_id: i64,
    pub audiobook_id: i64,
    pub position: i32,
    pub timestamp: NaiveDateTime,
}
