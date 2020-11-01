use style::render_tree::RenderNodeRef;
use super::box_model::Dimensions;

/// LayoutBox for the layout tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub box_type: BoxType,
    pub render_node: Option<RenderNodeRef>,
    pub dimensions: Dimensions,
    pub children: Vec<LayoutBox>,
    pub fmt_context: Option<FormattingContext>
}

/// Formatting context of each box
#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContext {
    Block,
    Inline
}

/// Different box types for each layout box
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    Anonymous
}

impl LayoutBox {
    pub fn new(node: Option<RenderNodeRef>, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: node,
            dimensions: Dimensions::default(),
            children: Vec::new(),
            fmt_context: None
        }
    }

    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    pub fn set_formatting_context(&mut self, ctx: FormattingContext) {
        self.fmt_context = Some(ctx);
    }
}

