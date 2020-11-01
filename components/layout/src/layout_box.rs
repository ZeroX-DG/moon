use super::box_model::Dimensions;
use style::render_tree::RenderNodeRef;

/// LayoutBox for the layout tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub box_type: BoxType,
    pub render_node: RenderNodeRef,
    pub position: BoxPosition,
    pub dimensions: Dimensions,
    pub children: Vec<LayoutBox>,
    pub parent_fmt_context: Option<FormattingContext>,
    pub fmt_context: Option<FormattingContext>,
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
    Block,
    Inline,
    Anonymous,
}

/// Position of a layout box
#[derive(Debug, Clone, PartialEq)]
pub struct BoxPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for BoxPosition {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl LayoutBox {
    pub fn new(node: RenderNodeRef, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: node,
            position: BoxPosition::default(),
            dimensions: Dimensions::default(),
            children: Vec::new(),
            parent_fmt_context: None,
            fmt_context: None,
        }
    }

    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    pub fn set_formatting_context(&mut self, ctx: FormattingContext) {
        self.fmt_context = Some(ctx);
    }

    pub fn set_parent_formatting_context(&mut self, ctx: FormattingContext) {
        self.parent_fmt_context = Some(ctx);
    }
}
