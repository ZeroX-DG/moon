/// This module is responsible for the box generation
/// of elements in the render tree. In other words,
/// this module transforms render tree to layout tree
/// to prepare for layouting process.
use super::layout_box::{LayoutBox, BoxType};
use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::{Display, OuterDisplayType, InnerDisplayType};

pub struct TreeBuilder<'a> {
    parent_stack: Vec<&'a LayoutBox>
}

impl<'a> TreeBuilder<'a> {
    pub fn new() -> Self {
        Self {
            parent_stack: Vec::new()
        }
    }

    pub fn build_layout_tree(&mut self, node: RenderNodeRef) -> Option<LayoutBox> {
        let layout_box = build_box_by_display(&node);

        if let Some(lb) = layout_box {
            for child in node.borrow().children {
                self.parent_stack.push(&lb);
                self.build_layout_tree(child.clone());
                self.parent_stack.pop();
            }
        }
        
        layout_box
    }
}

fn build_box_by_display(node: &RenderNodeRef) -> Option<LayoutBox> {
    let display = node.borrow().get_style(&Property::Display);

    match display.inner() {
        Value::Display(d) => match d {
            Display::Full(outer, inner) => match (outer, inner) {
                (OuterDisplayType::Block, InnerDisplayType::Flow) => {
                    let layout_box = LayoutBox::new(node.clone(), BoxType::Block);
                    Some(layout_box)
                }
                (OuterDisplayType::Inline, InnerDisplayType::Flow) => {
                    let layout_box = LayoutBox::new(node.clone(), BoxType::Inline);
                    Some(layout_box)
                }
                _ => None
            }
            _ => {
                log::warn!("Unsupport display type: {:#?}", d);
                None
            }
        }
        _ => unreachable!()
    }
}
