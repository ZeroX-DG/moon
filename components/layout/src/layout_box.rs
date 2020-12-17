/// This module contains the definition of
/// the layout box, which is the component
/// that made up the layout tree.
use super::box_model::Dimensions;
use style::render_tree::RenderNodeRef;

/// LayoutBox for the layout tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Type of this box (inline | block | anonymous)
    pub box_type: BoxType,

    /// Box model dimensions for this box
    pub dimensions: Dimensions,

    /// The render node that generate this box
    pub render_node: RenderNodeRef,

    /// The formatting context that this block establish
    pub formatting_context: Option<FormattingContext>,

    /// The children of this box
    pub children: Vec<LayoutBox>,
}

/// Formatting context of each box
#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContext {
    Block,
    Inline,
}

/// Different box types for each layout box
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    /// Block-level box
    Block,

    /// Inline-level box
    Inline,

    /// Anonymous inline / Anonymous block box
    /// depending on the formatting context of
    /// the parent box
    Anonymous,
}

impl LayoutBox {
    pub fn new(node: RenderNodeRef, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: node,
            dimensions: Dimensions::default(),
            formatting_context: None,
            children: Vec::new(),
        }
    }

    pub fn box_model(&mut self) -> &mut Dimensions {
        &mut self.dimensions
    }

    pub fn to_string(&self) -> String {
        dump_layout_tree(&self, 0)
    }
}

fn dump_layout_tree(root: &LayoutBox, level: usize) -> String {
    let mut result = String::new();
    let child_nodes = &root.children;
    result.push_str(&format!(
        "{}{:#?}({:#?})(x: {} | y: {} | width: {} | height: {})\n",
        "    ".repeat(level),
        root.box_type,
        root.render_node.borrow().node,
        root.dimensions.content.x,
        root.dimensions.content.y,
        root.dimensions.content.width,
        root.dimensions.content.height
    ));
    for node in child_nodes {
        result.push_str(&dump_layout_tree(node, level + 1));
    }
    return result;
}
