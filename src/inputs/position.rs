use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct InputPositionUpdate {
    pub chapter_index: u32,
    pub chapter_position: u64,
    pub client_timestamp: i64,
}
