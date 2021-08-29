use std::{
    any::Any,
    fmt::Debug,
};

use style::{render_tree::RenderNodeRef, value_processing::{Property, Value}, values::position::Position};

use crate::box_model::Dimensions;
use tree::idtree::{Tree, TreeNode, TreeNodeId};

pub type LayoutTree = Tree<Box<dyn LayoutBox>>;
pub type LayoutNode = TreeNode<Box<dyn LayoutBox>>;
pub type LayoutNodeId = TreeNodeId;

pub trait LayoutBox: Any + Debug {
    fn is_block(&self) -> bool;
    fn is_inline(&self) -> bool;
    fn render_node(&self) -> Option<RenderNodeRef>;
    fn is_anonymous(&self) -> bool {
        self.render_node().is_none()
    }
    fn friendly_name(&self) -> &str;
    fn dimensions(&self) -> &Dimensions;
    fn dimensions_mut(&mut self) -> &mut Dimensions;
    fn is_positioned(&self, position: Position) -> bool {
        match self.render_node() {
            Some(node) => match node.borrow().get_style(&Property::Position).inner() {
                Value::Position(pos) => *pos == position,
                _ => false
            }
            _ => false
        }
    }
    fn is_non_replaced(&self) -> bool {
        match &self.render_node() {
            Some(node) => match node.borrow().node.borrow().as_element_opt() {
                Some(e) => match e.tag_name().as_str() {
                    "video" | "image" | "img" | "canvas" => false,
                    _ => true,
                },
                _ => true,
            },
            _ => true,
        }
    }
    fn is_style_auto(&self, property: &Property) -> bool {
        if let Some(node) = &self.render_node() {
            let style = node.borrow().get_style(property);

            return style.is_auto();
        }
        return true;
    }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub fn children_are_inline(tree: &LayoutTree, node_id: &LayoutNodeId) -> bool {
    tree.children(node_id)
        .iter()
        .map(|child| tree.get_node(child))
        .all(|child| child.is_inline())
}

pub fn get_containing_block(tree: &LayoutTree, node_id: &LayoutNodeId) -> LayoutNodeId {
    tree.parent(node_id).unwrap().id()
}