/// This module is responsible for the box generation
/// of elements in the render tree. In other words,
/// this module transforms render tree to layout tree
/// to prepare for layouting process.
use super::layout_box::{LayoutBox, BoxType};
use style::render_tree::RenderNodeRef;
use std::rc::Rc;

pub struct TreeBuilder {
    parent_stack: Vec<Rc<LayoutBox>>
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            parent_stack: Vec::new()
        }
    }

    pub fn build_layout_tree(&self, node: RenderNodeRef) -> Option<LayoutBox> {
        let layout_box = build_box_by_display(&node);

        for child in node.borrow().children {
            self.build_layout_tree(child.clone());
        }

        layout_box
    }
}

fn build_box_by_display(node: &RenderNodeRef) -> Option<LayoutBox> {
    return None
}
