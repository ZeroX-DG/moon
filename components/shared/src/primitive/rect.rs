use serde::{Deserialize, Serialize};
use super::edge::EdgeSizes;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

impl Rect {
    pub fn add_outer_edges(&self, edges: &EdgeSizes) -> Self {
        Self {
            x: self.x - edges.left,
            y: self.y - edges.top,
            width: self.width + edges.left + edges.right,
            height: self.height + edges.top + edges.bottom,
        }
    }

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height
        }
    }
}

impl Into<(f32, f32, f32, f32)> for Rect {
    fn into(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}


