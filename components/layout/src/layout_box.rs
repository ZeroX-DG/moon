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
    pub render_node: Option<RenderNodeRef>,

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
        Some(s) => format!("{:?} Formatting Context", s),
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
