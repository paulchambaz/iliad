use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct InputPositionUpdate {
    pub chapter_index: i64,
    pub chapter_position: i64,
    pub timestamp: i64,
}
