/// This module contains the definition
/// for the box-model of each box in the
/// layout tree.

/// Box-model dimensions for each layout box
#[derive(Debug, Clone)]
pub struct Dimensions {
    pub content: ContentArea,
    pub padding: EdgeSizes,
    pub margin: EdgeSizes,
    pub border: EdgeSizes,
}

/// Size of the content area (all in px)
#[derive(Debug, Clone)]
pub struct ContentArea {
    pub width: f32,
    pub height: f32,
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
    Bottom
}

pub enum BoxComponent {
    Padding,
    Margin,
    Border
}

impl Dimensions {
    pub fn set_width(&mut self, width: f32) {
        self.content.width = width;
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

    pub fn margin_box_height(&self) -> f32 {
        self.content.height
            + self.padding.top
            + self.padding.bottom
            + self.border.top
            + self.border.bottom
            + self.margin.top
            + self.margin.bottom
    }

    pub fn padding_box_height(&self) -> f32 {
        self.content.height
            + self.padding.top
            + self.padding.bottom
            + self.border.top
            + self.border.bottom
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            content: ContentArea {
                width: 0.0,
                height: 0.0,
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
