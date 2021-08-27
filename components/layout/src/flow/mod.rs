use style::render_tree::RenderNodeRef;

use super::layout_box::LayoutBox;

#[derive(Debug)]
pub struct BlockBox {
    node: Option<RenderNodeRef>
}

impl LayoutBox for BlockBox {
    fn is_inline(&self) -> bool {
        false
    }

    fn is_block(&self) -> bool {
        true
    }

    fn render_node(&self) -> Option<RenderNodeRef> {
        self.node.clone()
    }
}

impl BlockBox {
    pub fn new() -> Self {
        Self {
            node: None
        }
    }

    pub fn new_anonymous() -> Self {
        Self {
            node: None
        }
    }
}
