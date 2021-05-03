use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderedBitmap {
    pub data: Vec<u8>
}
