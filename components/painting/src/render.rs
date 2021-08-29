use crate::command::DisplayCommand;
use layout::layout_box::{LayoutNode, LayoutNodeId, LayoutTree};

pub type PaintFn = dyn Fn(&LayoutNode) -> Option<DisplayCommand>;
pub type DisplayList = Vec<DisplayCommand>;

pub struct PaintChain<'a> {
    chain: Vec<&'a PaintFn>,
    layout_tree: &'a LayoutTree
}

pub struct PaintChainBuilder<'a> {
    paint_fns: Vec<&'a PaintFn>,
}

impl<'a> PaintChain<'a> {
    pub fn paint(&self, layout_node_id: &LayoutNodeId) -> DisplayList {
        let mut result = Vec::new();

        let node = self.layout_tree.get_node(layout_node_id);

        for paint_fn in &self.chain {
            if let Some(command) = paint_fn(node) {
                result.push(command);
            }
        }

        for child in self.layout_tree.children(layout_node_id) {
            result.extend(self.paint(child));
        }

        result
    }
}

impl<'a> PaintChainBuilder<'a> {
    pub fn new_chain() -> Self {
        Self {
            paint_fns: Vec::new(),
        }
    }

    pub fn with_function(mut self, paint_fn: &'a PaintFn) -> Self {
        self.paint_fns.push(paint_fn);
        self
    }

    pub fn build(self, layout_tree: &'a LayoutTree) -> PaintChain<'a> {
        PaintChain {
            chain: self.paint_fns,
            layout_tree
        }
    }
}
