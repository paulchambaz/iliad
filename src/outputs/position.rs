use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct OutputPositionUpdate {
    pub chapter_index: i64,
    pub chapter_position: i64,
    pub timestamp: i64,
}
