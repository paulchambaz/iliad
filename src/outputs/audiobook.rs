use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AudiobookShort {
    pub hash: String,
    pub title: String,
    pub author: String,
    pub archive_ready: bool,
}

#[derive(Serialize, Debug)]
pub struct AudiobookLong {
    pub hash: String,
    pub title: String,
    pub author: String,
    pub date: i32,
    pub description: String,
    pub genres: Vec<String>,
    pub duration: i64,
    pub size: i64,
    pub archive_ready: bool,
}
