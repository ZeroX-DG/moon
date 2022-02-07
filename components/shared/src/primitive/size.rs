use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width, height
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::new(0., 0.)
    }
}
