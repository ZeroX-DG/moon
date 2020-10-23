use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::length::Length;
use std::borrow::Borrow;

pub struct LayoutBox {
    pub render_node: RenderNodeRef,
    pub dimensions: Dimensions,
    pub children: Box<LayoutBox>,
}

pub struct Dimensions {
    content: Rect,
    padding: EdgeSizes,
    margin: EdgeSizes,
    border: EdgeSizes
}

/// Rect of the content area (all in px)
pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32
}

/// Edge size of the box (all in px)
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32
}
