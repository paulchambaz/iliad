use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Audiobook {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub file_path: String,
    pub hash: String,
}
