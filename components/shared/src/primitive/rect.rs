use super::{edge::EdgeSizes, Point, Size};
use serde::{Deserialize, Serialize};

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

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }

    /// Compute intersection with another rectangle. Modify the current rectangle into the
    /// intersection rectangle.
    pub fn intersect(&mut self, other: &Rect) {
        let x = f32::max(self.x, other.x);
        let y = f32::max(self.y, other.y);

        let w = f32::min(self.x + self.width, other.x + other.width) - x;
        let h = f32::min(self.y + self.height, other.y + other.height) - y;

        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
    }

    pub fn is_overlap_rect(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn is_contain_point(&self, point: &Point) -> bool {
        self.x <= point.x
            && self.x + self.width >= point.x
            && self.y <= point.y
            && self.y + self.height >= point.y
    }

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<(Point, Size)> for Rect {
    fn from((location, size): (Point, Size)) -> Self {
        Self {
            x: location.x,
            y: location.y,
            width: size.width,
            height: size.height,
        }
    }
}

impl Into<(f32, f32, f32, f32)> for Rect {
    fn into(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}
