use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x, y
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::new(0., 0.)
    }
}
