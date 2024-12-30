use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AudiobookShort {
    pub hash: String,
    pub title: String,
    pub author: String,
}

#[derive(Serialize, Debug)]
pub struct AudiobookLong {
    pub hash: String,
    pub title: String,
    pub author: String,
    pub date: i32,
    pub description: String,
    pub genres: Vec<String>,
    pub duration: u64,
    pub size: u64,
}
