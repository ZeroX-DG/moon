use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderedBitmap {
    pub data: Vec<u8>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SynParams {
    pub id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadFileContentParams {
    pub content: String,
    pub content_type: String
}

