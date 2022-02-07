pub enum Edge {
    Top,
    Left,
    Right,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for EdgeSizes {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

impl EdgeSizes {
    pub fn add_edge_sizes(&self, other: &EdgeSizes) -> Self {
        Self {
            top: self.top + other.top,
            right: self.right + other.right,
            bottom: self.bottom + other.bottom,
            left: self.left + other.left,
        }
    }
}
