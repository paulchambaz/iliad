use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct OutputPositionUpdate {
    pub chapter_index: u32,
    pub chapter_position: u64,
}
