/// This module contains the definition of
/// the layout box, which is the component
/// that made up the layout tree.
use super::box_model::Dimensions;
use super::formatting_context::FormattingContext;
use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
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

    /// The formatting context that this block establish
    pub formatting_context: Option<FormattingContext>,

    /// The children of this box
    pub children: Vec<LayoutBox>,
}

/// Different box types for each layout box
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    /// Block-level box
    Block,

    /// Inline-level box
    Inline
}

impl LayoutBox {
    pub fn new(node: RenderNodeRef, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: Some(node),
            dimensions: Dimensions::default(),
            formatting_context: None,
            children: Vec::new(),
        }
    }

    pub fn new_anonymous(box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: None,
            dimensions: Dimensions::default(),
            formatting_context: None,
            children: Vec::new()
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
                _ => true
            },
            _ => false
        }
    }

    pub fn is_non_replaced(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().node.borrow().as_element() {
                Some(e) => match e.tag_name().as_str() {
                    "video" | "image" | "img" | "canvas" => false,
                    _ => true
                },
                _ => true
            }
            _ => true
        }
    }

    // TODO: change to the correct behavior of inline block
    pub fn is_inline_block(&self) -> bool {
        self.is_inline()
    }

    // TODO: change to the correct behavior to detect normal flow
    pub fn is_in_normal_flow(&self) -> bool {
        true
    }

    pub fn is_absolutely_positioned(&self) -> bool {
        match &self.render_node {
            Some(node) => match node.borrow().get_style(&Property::Position).inner() {
                Value::Position(Position::Absolute) => true,
                _ => false
            },
            _ => false
        }
    }

    pub fn box_model(&mut self) -> &mut Dimensions {
        &mut self.dimensions
    }

    pub fn to_string(&self) -> String {
        dump_layout_tree(&self, 0)
    }

    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    pub fn set_formatting_context(&mut self, context: FormattingContext) {
        self.formatting_context = Some(context);
    }
}

fn dump_layout_tree(root: &LayoutBox, level: usize) -> String {
    let mut result = String::new();
    let child_nodes = &root.children;

    let formatting_context = match &root.formatting_context {
        Some(s) => format!("{:?}", s),
        None => format!("no formatting context")
    };

    if let Some(node) = &root.render_node {
        result.push_str(&format!(
            "{}[{:?}] {:#?} establish {}\n",
            "  ".repeat(level),
            root.box_type,
            node.borrow().node,
            formatting_context
        ));
    }
    else {
        result.push_str(&format!(
            "{}[Anonymous {:?}] establish {}\n",
            "  ".repeat(level),
            root.box_type,
            formatting_context
        ));
    }

    
    for node in child_nodes {
        result.push_str(&dump_layout_tree(node, level + 1));
    }
    return result;
}
