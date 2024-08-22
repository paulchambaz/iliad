use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Audiobook {
    pub hash: String, // primary key
    pub title: String,
    pub author: String,
    pub date: i32,
    pub description: String,
    pub genres: String,
    pub duration: u64,
    pub size: u64,
    pub path: String,
}
