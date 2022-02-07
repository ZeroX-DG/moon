use shared::primitive::*;

/// Box-model dimensions for each layout box
#[derive(Debug, Clone)]
pub struct BoxModel {
    pub padding: EdgeSizes,
    pub margin: EdgeSizes,
    pub border: EdgeSizes,
    pub offset: EdgeSizes
}

pub enum BoxComponent {
    Padding,
    Margin,
    Border,
}

impl BoxModel {
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
    pub fn margin_box(&self) -> EdgeSizes {
        self.padding
            .add_edge_sizes(&self.border)
            .add_edge_sizes(&self.margin)
    }

    pub fn padding_box(&self) -> EdgeSizes {
        self.padding.clone()
    }

    pub fn border_box(&self) -> EdgeSizes {
        self.padding.add_edge_sizes(&self.border)
    }
}

impl Default for BoxModel {
    fn default() -> Self {
        Self {
            padding: Default::default(),
            border: Default::default(),
            margin: Default::default(),
            offset: Default::default()
        }
    }
}
