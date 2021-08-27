/// Box-model dimensions for each layout box
#[derive(Debug, Clone)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub margin: EdgeSizes,
    pub border: EdgeSizes,
}

/// Size of the content area (all in px)
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

/// Edge size of the box (all in px)
#[derive(Debug, Clone)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

pub enum Edge {
    Top,
    Left,
    Right,
    Bottom,
}

pub enum BoxComponent {
    Padding,
    Margin,
    Border,
}

impl Dimensions {
    pub fn set_width(&mut self, width: f32) {
        self.content.width = width;
    }

    pub fn set_height(&mut self, height: f32) {
        self.content.height = height;
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.content.x = x;
        self.content.y = y;
    }

    pub fn set(&mut self, component: BoxComponent, edge: Edge, value: f32) {
        match component {
            BoxComponent::Margin => match edge {
                Edge::Top => self.margin.top = value,
                Edge::Right => self.margin.right = value,
                Edge::Bottom => self.margin.bottom = value,
                Edge::Left => self.margin.left = value,
            },
            BoxComponent::Padding => match edge {
                Edge::Top => self.padding.top = value,
                Edge::Right => self.padding.right = value,
                Edge::Bottom => self.padding.bottom = value,
                Edge::Left => self.padding.left = value,
            },
            BoxComponent::Border => match edge {
                Edge::Top => self.border.top = value,
                Edge::Right => self.border.right = value,
                Edge::Bottom => self.border.bottom = value,
                Edge::Left => self.border.left = value,
            },
        }
    }

    // we might need to review this for collapse margin
    // https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Box_Model/Mastering_margin_collapsing
    pub fn margin_box(&self) -> Rect {
        self.content
            .add_outer_edges(&self.padding)
            .add_outer_edges(&self.border)
            .add_outer_edges(&self.margin)
    }

    pub fn padding_box(&self) -> Rect {
        self.content.add_outer_edges(&self.padding)
    }

    pub fn border_box(&self) -> Rect {
        self.content
            .add_outer_edges(&self.padding)
            .add_outer_edges(&self.border)
    }

    pub fn content_box(&self) -> Rect {
        self.content.clone()
    }
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
}

impl Into<(f32, f32, f32, f32)> for Rect {
    fn into(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            content: Rect {
                width: 0.0,
                height: 0.0,
                x: 0.0,
                y: 0.0,
            },
            padding: Default::default(),
            border: Default::default(),
            margin: Default::default(),
        }
    }
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