use style::render_tree::RenderNodeRef;

use crate::layout_box::LayoutBox;

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

    fn friendly_name(&self) -> &str {
        "BlockBox"
    }
}

impl BlockBox {
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
