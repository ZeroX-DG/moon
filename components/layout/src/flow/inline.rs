use style::render_tree::RenderNodeRef;

use crate::layout_box::LayoutBox;

#[derive(Debug)]
pub struct InlineBox {
    node: Option<RenderNodeRef>
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
}

impl InlineBox {
    pub fn new(node: RenderNodeRef) -> Self {
        Self {
            node: Some(node)
        }
    }

    pub fn new_anonymous() -> Self {
        Self {
            node: None
        }
    }
}
