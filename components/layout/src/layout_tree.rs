use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::length::Length;
use std::borrow::Borrow;

pub struct LayoutBox {
    pub box_type: BoxType,
    pub render_node: RenderNodeRef,
    pub dimensions: Dimensions,
    pub children: Vec<LayoutBox>,
}

pub enum BoxType {
    Block,
    Inline,
    Anonymous
}

pub struct Dimensions {
    content: ContentSize,
    padding: EdgeSizes,
    margin: EdgeSizes,
    border: EdgeSizes
}

/// Size of the content area (all in px)
pub struct ContentSize {
    pub width: f32,
    pub height: f32
}

/// Edge size of the box (all in px)
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            content: ContentSize {
                width: 0.0,
                height: 0.0
            },
            padding: Default::default(),
            border: Default::default(),
            margin: Default::default()
        }
    }
}

impl Default for EdgeSizes {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0 
        }
    }
}

impl LayoutBox {
    pub fn new(node: RenderNodeRef) -> Self {
        Self {
            box_type: BoxType::Inline,
            children: Vec::new(),
            render_node: node,
            dimensions: Default::default()
        }
    }
}

pub fn build_layout_tree_from_node(render_node: RenderNodeRef) -> LayoutBox {
    let layout_box = LayoutBox::new(render_node.clone());
    layout_box
}
