/// This module contains the definition of
/// the layout box, which is the component
/// that made up the layout tree.
use super::box_model::Dimensions;
use super::line_box::LineBox;
use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::{Display, InnerDisplayType};
use style::values::float::Float;
use style::values::position::Position;

/// LayoutBox for the layout tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Type of this box (inline | block | anonymous)
    pub box_type: BoxType,

    /// Box model dimensions for this box
    pub dimensions: Dimensions,

    /// The render node that generate this box
    pub render_node: Option<RenderNodeRef>,

    /// Indicate if this box only contain inline
    pub children_are_inline: bool,

    /// The children of this box
    pub children: Vec<LayoutBox>,

    /// The line boxes of this layout box
    pub line_boxes: Vec<LineBox>,
}

/// Different box types for each layout box
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    /// Block-level box
    Block,

    /// Inline-level box
    Inline,
}

impl LayoutBox {
    pub fn new(node: RenderNodeRef, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: Some(node),
            dimensions: Dimensions::default(),
            children_are_inline: false,
            children: Vec::new(),
            line_boxes: Vec::new(),
        }
    }

    pub fn new_anonymous(box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: None,
            dimensions: Dimensions::default(),
            children_are_inline: false,
            children: Vec::new(),
            line_boxes: Vec::new(),
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.render_node.is_none()
    }

    pub fn is_inline(&self) -> bool {
        self.box_type == BoxType::Inline
    }

    pub fn is_block(&self) -> bool {
        self.box_type == BoxType::Block
    }

    pub fn is_float(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().get_style(&Property::Float).inner() {
                Value::Float(Float::None) => false,
                _ => true,
            },
            _ => false,
        }
    }

    pub fn is_non_replaced(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().node.borrow().as_element() {
                Some(e) => match e.tag_name().as_str() {
                    "video" | "image" | "img" | "canvas" => false,
                    _ => true,
                },
                _ => true,
            },
            _ => true,
        }
    }

    pub fn is_inline_block(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().get_style(&Property::Display).inner() {
                Value::Display(Display::Full(_, InnerDisplayType::FlowRoot)) => self.is_inline(),
                _ => false,
            },
            _ => false,
        }
    }

    // TODO: change to the correct behavior to detect normal flow
    pub fn is_in_normal_flow(&self) -> bool {
        true
    }

    pub fn is_absolutely_positioned(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().get_style(&Property::Position).inner() {
                Value::Position(Position::Absolute) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn box_model(&mut self) -> &mut Dimensions {
        &mut self.dimensions
    }

    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    pub fn set_children_inline(&mut self, value: bool) {
        self.children_are_inline = value;
    }

    pub fn children_are_inline(&self) -> bool {
        self.children_are_inline
    }

    pub fn is_height_auto(&self) -> bool {
        if let Some(node) = &self.render_node {
            let computed_height = node.borrow().get_style(&Property::Height);

            return computed_height.is_auto();
        }
        return true;
    }
}
