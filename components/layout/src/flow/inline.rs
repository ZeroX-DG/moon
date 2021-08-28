use std::any::Any;

use style::render_tree::RenderNodeRef;

use crate::{box_model::Dimensions, layout_box::LayoutBox};

#[derive(Debug)]
pub struct InlineBox {
    node: Option<RenderNodeRef>,
    dimensions: Dimensions,
}

impl LayoutBox for InlineBox {
    fn is_inline(&self) -> bool {
        true
    }

    fn is_block(&self) -> bool {
        false
    }

    fn render_node(&self) -> Option<RenderNodeRef> {
        self.node.clone()
    }

    fn friendly_name(&self) -> &str {
        "InlineBox"
    }

    fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }

    fn dimensions_mut(&mut self) -> &mut Dimensions {
        &mut self.dimensions
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl InlineBox {
    pub fn new(node: RenderNodeRef) -> Self {
        Self {
            node: Some(node),
            dimensions: Default::default()
        }
    }

    pub fn new_anonymous() -> Self {
        Self {
            node: None,
            dimensions: Default::default()
        }
    }
}
