use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Audiobook {
    pub hash: String, // primary key
    pub title: String,
    pub author: String,
    pub date: i32,
    pub description: String,
    pub genres: String,
    pub duration: i64,
    pub size: i64,
    pub path: String,
    pub final_chapter_index: i64,
    pub final_chapter_position: i64,
    pub cover: Option<String>,
    pub archive_ready: bool,
}
